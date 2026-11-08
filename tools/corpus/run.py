#!/usr/bin/env python3
"""Round-trip a corpus of real-world Excel files through gridcraft-cli and report how it went.

    python3 tools/corpus/run.py --bin target/release/gridcraft-cli --corpus DIR --out DIR
                                [--timeout 30] [--jobs N] [--soffice] [--deadline EPOCH] [--limit N]

DIR (from fetch.sh) holds the files and manifest.tsv. For every file, each step in its own process
with a timeout (commands from apps/gridcraft-cli/src/main.rs):

  open           gridcraft-cli run --in <file> --cmd document.inspect
                 open + sheets, used ranges, cell counts, tables, charts, names...
  values         gridcraft-cli run --in <file> --cmd 'sheet.read={"sheet":i}'
                                               --cmd 'sheet.read={"sheet":i,"formulas":true}' ...
                 every sheet's values and formulas (sheets over 1,000,000 cells are left out)
  save           gridcraft-cli convert <file> <rt.xlsx>      open + save as .xlsx
  reopen         open and values again, on the saved file
  check          gridcraft-cli check <file> --json --max-report 3
                 the CLI's own cached-value calc oracle and XLSX round-trip comparison

Then, in Python only: the saved file is a valid zip whose XML parts all parse; cells with a
value and cells with a formula counted in the worksheet XML before and after; and with
--soffice, LibreOffice's CSV of every sheet of both files. Each step ends ok, err (non-zero
exit), panic (Rust panic, exit 101), crash (killed by a signal) or timeout. Writes
corpus-report.json and corpus-report.md to --out and appends the summary to
$GITHUB_STEP_SUMMARY when set. Standard library only.
"""

from __future__ import annotations

import argparse
import collections
import concurrent.futures
import csv
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
import zipfile
import xml.etree.ElementTree as ET

TOOL = "gridcraft-cli"
EXT = ".xlsx"
S = "{http://schemas.openxmlformats.org/spreadsheetml/2006/main}"
S_STRICT = "{http://purl.oclc.org/ooxml/spreadsheetml/main}"
OFFICE_DOC = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument"
STRICT_OFFICE_DOC = "http://purl.oclc.org/ooxml/officeDocument/relationships/officeDocument"
WORKSHEET_REL = ("/relationships/worksheet",)
MAX_XML_BYTES = 200 * 1024 * 1024  # unpacked part size above which the part is not parsed
MAX_READ_CELLS = 1_000_000  # sheet.read refuses larger ranges (crates/engine/src/cmd/inspect.rs)
MAX_TOTAL_CELLS = 3_000_000  # above this the values step is skipped for the whole file
STAGES = ("open", "values", "save", "reopen", "reopen_values", "package")
STAGE_RANK = {s: i for i, s in enumerate(STAGES + ("check",))}
# Per-sheet counts in document.inspect compared before and after.
INSPECT_LISTS = ("merges", "tables", "charts")
INSPECT_COUNTS = ("images", "shapes", "conditionalFormats", "validations", "comments")


# ---------------------------------------------------------------- processes


# A file that makes the app allocate without bound must fail on its own (out of memory shows up
# as a crash) instead of taking the whole runner down with it. LibreOffice is left unlimited:
# it reserves a lot of address space it never touches.
MEM_LIMIT_BYTES = int(os.environ.get("CORPUS_MEM_LIMIT_MB", "2500")) * 1024 * 1024


def _limited(argv: list[str]) -> list[str]:
    # prlimit (util-linux) instead of preexec_fn: the runner forks from worker threads.
    if "soffice" in os.path.basename(argv[0]) or not shutil.which("prlimit"):
        return argv
    return ["prlimit", f"--as={MEM_LIMIT_BYTES}", "--", *argv]


def run_proc(argv: list[str], timeout: float, env: dict | None = None, cwd: str | None = None) -> dict:
    """Run argv in its own process group; kill the whole group on timeout."""
    t0 = time.monotonic()
    try:
        p = subprocess.Popen(
            _limited(argv),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            stdin=subprocess.DEVNULL,
            env=env,
            cwd=cwd,
            start_new_session=True,
        )
    except OSError as e:
        return {"status": "err", "code": None, "stdout": b"", "stderr": str(e), "seconds": 0.0}
    try:
        out, err = p.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(p.pid, signal.SIGKILL)
        except OSError:
            pass
        out, err = p.communicate()
        return {"status": "timeout", "code": None, "stdout": out, "stderr": err.decode("utf-8", "replace"), "seconds": round(time.monotonic() - t0, 2)}
    err_text = err.decode("utf-8", "replace")
    code = p.returncode
    if code == 0:
        status = "ok"
    elif code < 0:
        status = "crash"
    elif code == 101 or "panicked at" in err_text:
        status = "panic"
    else:
        status = "err"
    return {"status": status, "code": code, "stdout": out, "stderr": err_text, "seconds": round(time.monotonic() - t0, 2)}


def error_message(r: dict, paths: list[str]) -> str | None:
    """One line saying why a step failed, with file paths taken out."""
    if r["status"] == "ok":
        return None
    err = r["stderr"]
    for p in paths:
        if p:
            err = err.replace(p, "<file>")
    lines = [ln.strip() for ln in err.splitlines() if ln.strip()]
    if r["status"] == "timeout":
        return "timeout"
    if r["status"] == "crash":
        sig = -r["code"]
        try:
            name = signal.Signals(sig).name
        except ValueError:
            name = f"signal {sig}"
        tail = next((ln for ln in reversed(lines) if "overflow" in ln or "fatal" in ln.lower()), "")
        return f"killed by {name}" + (f": {tail}" if tail else "")
    if r["status"] == "panic":
        for i, ln in enumerate(lines):
            m = re.search(r"panicked at (\S+?):\d+:\d+:?\s*(.*)", ln)
            if m:
                msg = m.group(2) or (lines[i + 1] if i + 1 < len(lines) else "")
                return f"panic at {m.group(1)}: {msg}"
        return "panic: " + (lines[-1] if lines else "")
    ours = [ln for ln in lines if ln.startswith(f"{TOOL}: ")]
    other = [ln for ln in lines if not ln.startswith("Warning:")]
    msg = (ours or other or lines or [f"exit {r['code']}"])[0]
    return msg.removeprefix(f"{TOOL}: ")


def group_key(msg: str) -> str:
    """Error message with the variable bits (numbers, quoted names) folded, for grouping."""
    msg = re.sub(r"`[^`]*`", "`…`", msg)
    msg = re.sub(r"\"[^\"]*\"", '"…"', msg)
    msg = re.sub(r"\b0x[0-9a-fA-F]+\b", "N", msg)
    msg = re.sub(r"\d+", "N", msg)
    return msg[:200]


# ---------------------------------------------------------------- package checks


def main_part(z: zipfile.ZipFile) -> str | None:
    try:
        rels = ET.fromstring(z.read("_rels/.rels"))
    except (KeyError, ET.ParseError):
        return None
    for rel in rels:
        if rel.get("Type") in (OFFICE_DOC, STRICT_OFFICE_DOC):
            target = (rel.get("Target") or "").lstrip("/")
            return target
    return None


def check_package(path: str) -> dict:
    """Is `path` a zip whose XML parts all parse, with content types and a main document?"""
    out = {"zip": False, "content_types": False, "main_part": None, "xml_parts": 0, "xml_errors": [], "error": None}
    try:
        with zipfile.ZipFile(path) as z:
            bad = z.testzip()
            if bad is not None:
                out["error"] = f"bad CRC in {bad}"
                return out
            out["zip"] = True
            names = set(z.namelist())
            out["content_types"] = "[Content_Types].xml" in names
            out["main_part"] = main_part(z)
            for info in z.infolist():
                n = info.filename
                if not (n.endswith(".xml") or n.endswith(".rels")) or info.file_size > MAX_XML_BYTES:
                    continue
                out["xml_parts"] += 1
                try:
                    ET.fromstring(z.read(n))
                except ET.ParseError as e:
                    out["xml_errors"].append(f"{n}: {e}")
    except (zipfile.BadZipFile, OSError, RuntimeError, NotImplementedError, EOFError) as e:
        out["error"] = f"{type(e).__name__}: {e}"
        return out
    if out["main_part"] is None:
        out["error"] = "no officeDocument relationship"
    elif out["main_part"] not in names:
        out["error"] = f"main part {out['main_part']} missing"
    elif out["xml_errors"]:
        out["error"] = f"{len(out['xml_errors'])} XML part(s) do not parse"
    elif not out["content_types"]:
        out["error"] = "no [Content_Types].xml"
    return out


def xlsx_counts(path: str) -> dict | None:
    """Sheets, cells with a value and cells with a formula in the worksheet XML, read with the
    standard library. None when the file is not a readable package (encrypted, broken zip)."""
    try:
        with zipfile.ZipFile(path) as z:
            wb = main_part(z)
            if wb is None:
                return None
            folder = wb.rsplit("/", 1)[0] if "/" in wb else ""
            rels_name = f"{folder}/_rels/{wb.rsplit('/', 1)[-1]}.rels" if folder else f"_rels/{wb}.rels"
            rels = ET.fromstring(z.read(rels_name))
            targets = []
            for rel in rels:
                if (rel.get("Type") or "").endswith(WORKSHEET_REL) and rel.get("TargetMode") != "External":
                    t = rel.get("Target") or ""
                    t = t.lstrip("/") if t.startswith("/") else os.path.normpath(f"{folder}/{t}" if folder else t).replace(os.sep, "/")
                    targets.append(t)
            out = {"sheets": len(targets), "cells": 0, "formulas": 0}
            for t in targets:
                info = z.getinfo(t)
                if info.file_size > MAX_XML_BYTES:
                    return None
                with z.open(t) as f:
                    for _, el in ET.iterparse(f):
                        if el.tag in (S + "c", S_STRICT + "c"):
                            kids = {k.tag.rsplit("}", 1)[-1] for k in el}
                            if kids & {"v", "is"}:
                                out["cells"] += 1
                            if "f" in kids:
                                out["formulas"] += 1
                            el.clear()
                        elif el.tag in (S + "row", S_STRICT + "row"):
                            el.clear()
            return out
    except (zipfile.BadZipFile, KeyError, OSError, ET.ParseError, RuntimeError, NotImplementedError, EOFError, ValueError):
        return None


# ---------------------------------------------------------------- cell values

A1 = re.compile(r"^\$?([A-Za-z]{1,3})\$?(\d+)$")


def parse_cell(a1: str) -> tuple[int, int] | None:
    m = A1.match(a1)
    if not m:
        return None
    col = 0
    for ch in m.group(1).upper():
        col = col * 26 + ord(ch) - 64
    return int(m.group(2)), col


def range_area(a1: str | None) -> int:
    if not a1:
        return 0
    parts = a1.split("!")[-1].split(":")
    a = parse_cell(parts[0])
    b = parse_cell(parts[-1])
    if not a or not b:
        return 0
    return (abs(b[0] - a[0]) + 1) * (abs(b[1] - a[1]) + 1)


def norm(v) -> str | None:
    if v is None:
        return None
    if isinstance(v, bool):
        return "TRUE" if v else "FALSE"
    if isinstance(v, (int, float)):
        return format(float(v), ".12g")
    if isinstance(v, str):
        return v if v != "" else None
    if isinstance(v, dict) and "error" in v:
        return str(v["error"])
    return json.dumps(v, sort_keys=True, ensure_ascii=False)


def json_lines(stdout: bytes) -> list[dict]:
    out = []
    for line in stdout.decode("utf-8", "replace").splitlines():
        line = line.strip()
        if line.startswith("{"):
            try:
                out.append(json.loads(line))
            except json.JSONDecodeError:
                pass
    return out


def inspect_of(r: dict) -> dict | None:
    for obj in json_lines(r["stdout"]):
        if obj.get("command") == "document.inspect":
            return obj.get("result")
    return None


def inspect_summary(info: dict) -> dict:
    sheets = info.get("sheets") or []
    out = {"sheets": len(sheets), "cells": sum(int(s.get("cells") or 0) for s in sheets), "names": len(info.get("names") or [])}
    for k in INSPECT_LISTS:
        out[k] = sum(len(s.get(k) or []) for s in sheets)
    for k in INSPECT_COUNTS:
        out[k] = sum(int(s.get(k) or 0) for s in sheets)
    out["autofilters"] = sum(1 for s in sheets if s.get("autofilter"))
    out["hidden_sheets"] = sum(1 for s in sheets if (s.get("visibility") or "Visible") != "Visible")
    return out


def read_values(step, name: str, path: str, info: dict, bin_: str) -> dict | None:
    """Runs one process reading every sheet's values and formulas. Returns {"values": {key: v},
    "formulas": {key: text}, "skipped_sheets": n} or None when the step failed."""
    sheets = info.get("sheets") or []
    areas = [range_area(s.get("usedRange")) for s in sheets]
    if sum(a for a in areas if a <= MAX_READ_CELLS) > MAX_TOTAL_CELLS:
        return {"skipped": "too large"}
    argv = [bin_, "run", "--in", path]
    wanted = []
    for i, area in enumerate(areas):
        if 0 < area <= MAX_READ_CELLS:
            argv += ["--cmd", "sheet.read=" + json.dumps({"sheet": i}), "--cmd", "sheet.read=" + json.dumps({"sheet": i, "formulas": True})]
            wanted.append(i)
    if not wanted:  # nothing to read (empty workbook) or every sheet too large
        return {"skipped": "no readable sheet"} if any(areas) else {"values": {}, "formulas": {}, "skipped_sheets": 0}
    r = step(name, argv)
    if r["status"] != "ok":
        return None
    reads = [o.get("result") or {} for o in json_lines(r["stdout"]) if o.get("command") == "sheet.read"]
    values: dict = {}
    formulas: dict = {}
    for n, res in enumerate(reads):
        is_formulas = n % 2 == 1
        start = parse_cell(str(res.get("range") or "A1").split(":")[0]) or (1, 1)
        sheet = res.get("sheet")
        for ri, row in enumerate(res.get("values") or []):
            for ci, v in enumerate(row if isinstance(row, list) else [row]):
                key = (sheet, start[0] + ri, start[1] + ci)
                if is_formulas:
                    if isinstance(v, str) and v.startswith("=") and norm(values.get(key)) != v:
                        formulas[key] = v
                else:
                    nv = norm(v)
                    if nv is not None:
                        values[key] = nv
    return {"values": values, "formulas": formulas, "skipped_sheets": sum(1 for a in areas if a > MAX_READ_CELLS)}


def compare_values(a: dict, b: dict) -> dict:
    va, vb = a["values"], b["values"]
    fa, fb = a["formulas"], b["formulas"]
    union = set(va) | set(vb)
    diffs = [k for k in union if va.get(k) != vb.get(k)]
    fdiffs = [k for k in set(fa) | set(fb) if fa.get(k) != fb.get(k)]
    out = {
        "cells_before": len(va), "cells_after": len(vb), "value_diffs": len(diffs),
        "formulas_before": len(fa), "formulas_after": len(fb), "formula_diffs": len(fdiffs),
        "values_kept": round(1 - len(diffs) / len(union), 4) if union else 1.0,
    }
    ex = []
    for k in sorted(diffs, key=lambda k: (str(k[0]), k[1], k[2]))[:3]:
        ex.append(f"{k[0]}!R{k[1]}C{k[2]}: {va.get(k)!r} -> {vb.get(k)!r}"[:160])
    for k in sorted(fdiffs, key=lambda k: (str(k[0]), k[1], k[2]))[:3]:
        ex.append(f"{k[0]}!R{k[1]}C{k[2]}: {fa.get(k)!r} -> {fb.get(k)!r}"[:160])
    if ex:
        out["examples"] = ex
    return out


# ---------------------------------------------------------------- LibreOffice

_local = threading.local()
CSV_ALL_SHEETS = "csv:Text - txt - csv (StarCalc):44,34,76,1,,0,false,true,false,false,false,-1"


def soffice_cells(soffice: str, path: str, timeout: float) -> tuple[collections.Counter | None, str | None]:
    """Every non-empty cell LibreOffice shows on every sheet of `path`, as a multiset of text
    (each worker thread has its own LibreOffice profile)."""
    if not hasattr(_local, "profile"):
        _local.profile = tempfile.mkdtemp(prefix="lo-profile-")
    outdir = tempfile.mkdtemp(prefix="lo-out-")
    try:
        r = run_proc(
            [soffice, f"-env:UserInstallation=file://{_local.profile}", "--headless", "--norestore", "--nolockcheck",
             "--convert-to", CSV_ALL_SHEETS, "--outdir", outdir, path],
            timeout,
        )
        files = sorted(f for f in os.listdir(outdir) if f.endswith(".csv"))
        if r["status"] != "ok" or not files:
            return None, r["status"] if r["status"] != "ok" else "no output"
        cells: collections.Counter = collections.Counter()
        for name in files:
            with open(os.path.join(outdir, name), encoding="utf-8", errors="replace", newline="") as f:
                for row in csv.reader(f):
                    cells.update(c for c in row if c != "")
        return cells, None
    except csv.Error as e:
        return None, f"csv: {e}"
    finally:
        shutil.rmtree(outdir, ignore_errors=True)


def overlap(a: collections.Counter | None, b: collections.Counter | None) -> float | None:
    if a is None or b is None:
        return None
    na, nb = sum(a.values()), sum(b.values())
    if na == 0 and nb == 0:
        return 1.0
    return round(sum((a & b).values()) / max(na, nb), 4)


# ---------------------------------------------------------------- one file


def process(entry: dict, args, workdir: str, env: dict) -> dict:
    src = os.path.join(args.corpus, entry["file"])
    rec = {"file": entry["file"], "source": entry["source"], "url": entry["url"], "bytes": int(entry["bytes"])}
    rt = os.path.join(workdir, f"{entry['n']:05d}{EXT}")
    paths = [src, rt, args.corpus, workdir]

    def step(name: str, argv: list[str], timeout: float | None = None) -> dict:
        r = run_proc(argv, timeout or args.timeout, env)
        rec[name] = r["status"]
        rec[f"{name}_seconds"] = r["seconds"]
        msg = error_message(r, paths)
        if msg:
            rec[f"{name}_error"] = msg
        return r

    def open_and_read(open_name: str, values_name: str, path: str):
        """`open_name`: document.inspect; `values_name`: every sheet's values and formulas."""
        r = step(open_name, [args.bin, "run", "--in", path, "--cmd", "document.inspect"])
        info = inspect_of(r) if r["status"] == "ok" else None
        if r["status"] == "ok" and info is None:
            rec[open_name], rec[f"{open_name}_error"] = "err", "no document.inspect result on stdout"
        vals = read_values(step, values_name, path, info, args.bin) if info is not None else {"skipped": "not opened"}
        if vals is not None and "skipped" in vals:
            rec[values_name], rec[f"{values_name}_note"] = "skipped", vals["skipped"]
            vals = None
        elif vals is not None:
            rec.setdefault(values_name, "ok")  # an empty workbook needs no read
            if vals.get("skipped_sheets"):
                rec[f"{values_name}_note"] = f"{vals['skipped_sheets']} sheet(s) over {MAX_READ_CELLS} cells not read"
        return r, info, vals

    r_open, info_a, vals_a = open_and_read("open", "values", src)
    rec["open_warnings"] = sum(1 for ln in r_open["stderr"].splitlines() if ln.startswith("Warning:"))
    if info_a is not None:
        rec["inspect"] = inspect_summary(info_a)
    r_save = step("save", [args.bin, "convert", src, rt])
    saved = r_save["status"] == "ok" and os.path.isfile(rt) and os.path.getsize(rt) > 0
    if r_save["status"] == "ok" and not saved:
        rec["save"] = "err"
        rec["save_error"] = "convert exited 0 but wrote no file"
    info_b = vals_b = None
    if saved:
        rec["rt_bytes"] = os.path.getsize(rt)
        _, info_b, vals_b = open_and_read("reopen", "reopen_values", rt)
        pkg = check_package(rt)
        rec["package"] = "ok" if pkg["error"] is None else "err"
        if pkg["error"]:
            rec["package_error"] = pkg["error"]
            rec["package_xml_errors"] = pkg["xml_errors"][:5]
    else:
        for k in ("reopen", "reopen_values", "package"):
            rec[k] = "skipped"

    # Fidelity: the app's view before/after, then the raw worksheet XML before/after.
    if info_a is not None and info_b is not None:
        ia, ib = inspect_summary(info_a), inspect_summary(info_b)
        rec["count_changes"] = {k: [ia[k], ib[k]] for k in ia if ia[k] != ib[k]}
    if vals_a is not None and vals_b is not None:
        rec["values_cmp"] = compare_values(vals_a, vals_b)
        rec["values_kept"] = rec["values_cmp"]["values_kept"]
    xa = xlsx_counts(src)
    xb = xlsx_counts(rt) if saved else None
    if xa:
        rec["xml_before"] = xa
    if xb:
        rec["xml_after"] = xb
    if xa and xb:
        rec["xml_cells_kept"] = round(min(xa["cells"], xb["cells"]) / xa["cells"], 4) if xa["cells"] else 1.0
        rec["xml_formulas_kept"] = round(min(xa["formulas"], xb["formulas"]) / xa["formulas"], 4) if xa["formulas"] else 1.0
    if vals_a is not None and xa:
        # How many of the file's cells the app shows at all (no save involved).
        rec["open_cells_vs_xml"] = round(min(len(vals_a["values"]), xa["cells"]) / xa["cells"], 4) if xa["cells"] else 1.0

    # The CLI's own oracle: cached values vs GridCraft's recalculation, and its round-trip diff.
    if not args.no_check:
        r = run_proc([args.bin, "check", src, "--json", "--max-report", "3"], args.timeout * 2, env)
        rec["check_seconds"] = r["seconds"]
        report = None
        if r["status"] in ("ok", "err"):
            try:
                report = json.loads(r["stdout"].decode("utf-8", "replace"))
            except json.JSONDecodeError:
                report = None
        if report and report.get("files"):
            f = report["files"][0]
            c = f.get("calc") or {}
            rtp = f.get("roundTrip") or {}
            rec["check"] = "ok"
            rec["check_result"] = {
                "opened": f.get("opened"), "error": f.get("error"), "formula_cells": c.get("formulaCells"), "checked": c.get("checked"),
                "matched": c.get("matched"), "mismatched": c.get("mismatched"), "no_oracle": c.get("noOracle"),
                "round_trip_error": rtp.get("error"), "round_trip_differing": rtp.get("differing") or [],
                "signatures": c.get("signatures") or {},
            }
        else:
            rec["check"] = r["status"] if r["status"] != "ok" else "err"
            rec["check_error"] = error_message(r, paths) if r["status"] != "ok" else "no JSON report"
            if r["status"] == "err" and not rec["check_error"]:
                rec["check_error"] = "no JSON report"

    if args.soffice and saved:
        lo_a, ea = soffice_cells(args.soffice, src, args.timeout * 2)
        lo_b, eb = soffice_cells(args.soffice, rt, args.timeout * 2)
        rec["lo_cells_kept"] = overlap(lo_a, lo_b)
        if ea or eb:
            rec["lo_error"] = f"original: {ea or 'ok'}, round-trip: {eb or 'ok'}"
    if not args.keep:
        try:
            os.remove(rt)
        except OSError:
            pass
    rec["verdict"] = verdict(rec)
    return rec


FIDELITY_KEYS = ("values_kept", "xml_cells_kept", "xml_formulas_kept", "lo_cells_kept")


def verdict(rec: dict) -> str:
    for stage in STAGES:
        st = rec.get(stage)
        if st not in ("ok", "skipped", None):
            return f"{stage}:{st}"
    if rec.get("save") != "ok":
        return "save:skipped"
    if rec.get("check") in ("panic", "crash", "timeout"):
        return f"check:{rec['check']}"
    vals = [rec.get(k) for k in FIDELITY_KEYS if rec.get(k) is not None]
    if vals and min(vals) < 0.99:
        return "content_changed"
    if (rec.get("values_cmp") or {}).get("formula_diffs"):
        return "formulas_changed"
    return "ok"


def badness(rec: dict) -> tuple:
    v = rec["verdict"]
    if ":" in v:
        stage, st = v.split(":", 1)
        return (0, STAGE_RANK.get(stage, 9), {"panic": 0, "crash": 0, "timeout": 1, "err": 2}.get(st, 3), rec["file"])
    vals = [rec.get(k) for k in FIDELITY_KEYS if rec.get(k) is not None]
    return (1, min(vals) if vals else 1.0, -len(rec.get("count_changes") or {}), rec["file"])


# ---------------------------------------------------------------- report


def pct(n: int, d: int) -> str:
    return f"{100.0 * n / d:.1f}%" if d else "-"


def md_cell(s) -> str:
    return str(s).replace("|", "\\|").replace("\n", " ")[:220]


def summarize(recs: list[dict], not_run: int) -> dict:
    n = len(recs)
    s: dict = {"files": n + not_run, "processed": n, "not_run_deadline": not_run}
    for stage in STAGES + ("check",):
        s[stage] = dict(collections.Counter(r.get(stage, "skipped") for r in recs))
    s["verdicts"] = dict(collections.Counter(r["verdict"] for r in recs).most_common())
    s["full_round_trip_ok"] = sum(1 for r in recs if all(r.get(k) == "ok" for k in ("open", "save", "reopen", "package")))
    for key in FIDELITY_KEYS + ("open_cells_vs_xml",):
        vals = sorted(r[key] for r in recs if r.get(key) is not None)
        if vals:
            s[key] = {
                "files": len(vals),
                "identical": sum(1 for v in vals if v >= 0.9999),
                "ge_0.99": sum(1 for v in vals if v >= 0.99),
                "lt_0.90": sum(1 for v in vals if v < 0.90),
                "median": vals[len(vals) // 2],
                "mean": round(sum(vals) / len(vals), 4),
            }
    vc = [r["values_cmp"] for r in recs if r.get("values_cmp")]
    s["cell_values"] = {
        "files_compared": len(vc),
        "cells_before": sum(v["cells_before"] for v in vc),
        "cells_after": sum(v["cells_after"] for v in vc),
        "value_diffs": sum(v["value_diffs"] for v in vc),
        "formulas_before": sum(v["formulas_before"] for v in vc),
        "formulas_after": sum(v["formulas_after"] for v in vc),
        "formula_diffs": sum(v["formula_diffs"] for v in vc),
        "files_with_value_diffs": sum(1 for v in vc if v["value_diffs"]),
        "files_with_formula_diffs": sum(1 for v in vc if v["formula_diffs"]),
    }
    changes = collections.Counter()
    for r in recs:
        for label, (a, b) in (r.get("count_changes") or {}).items():
            changes[f"{label} {'lost' if b < a else 'gained'}"] += 1
    s["structure_changes"] = dict(changes.most_common())
    cr = [r["check_result"] for r in recs if r.get("check_result")]
    sigs = collections.Counter()
    differing = collections.Counter()
    for c in cr:
        sigs.update(c.get("signatures") or {})
        differing.update(c.get("round_trip_differing") or [])
    s["check"] = {
        "status": dict(collections.Counter(r.get("check", "skipped") for r in recs)),
        "files": len(cr),
        "opened": sum(1 for c in cr if c.get("opened")),
        "formula_cells": sum(c.get("formula_cells") or 0 for c in cr),
        "checked": sum(c.get("checked") or 0 for c in cr),
        "matched": sum(c.get("matched") or 0 for c in cr),
        "mismatched": sum(c.get("mismatched") or 0 for c in cr),
        "no_oracle": sum(c.get("no_oracle") or 0 for c in cr),
        "files_with_calc_mismatch": sum(1 for c in cr if c.get("mismatched")),
        "round_trip_errors": sum(1 for c in cr if c.get("round_trip_error")),
        "round_trip_clean": sum(1 for c in cr if c.get("opened") and not c.get("round_trip_error") and not c.get("round_trip_differing")),
        "categories_differing": dict(differing.most_common()),
        "top_signatures": dict(sigs.most_common(15)),
    }
    by_src = {}
    for src in sorted({r["source"] for r in recs}):
        rs = [r for r in recs if r["source"] == src]
        by_src[src] = {"files": len(rs), "open_ok": sum(r.get("open") == "ok" for r in rs), "round_trip_ok": sum(r["verdict"] == "ok" for r in rs)}
    s["by_source"] = by_src
    return s


def error_groups(recs: list[dict]) -> list[dict]:
    groups: dict = {}
    for r in recs:
        for stage in STAGES + ("check",):
            msg = r.get(f"{stage}_error")
            if not msg and stage == "check":
                c = r.get("check_result") or {}
                msg = c.get("error") and f"check could not open: {c['error']}" or c.get("round_trip_error") and f"check round trip: {c['round_trip_error']}"
            if not msg:
                continue
            key = (stage, r.get(stage), group_key(msg))
            g = groups.setdefault(key, {"stage": stage, "status": r.get(stage), "message": msg, "count": 0, "files": []})
            g["count"] += 1
            if len(g["files"]) < 5:
                g["files"].append(r["file"])
    return sorted(groups.values(), key=lambda g: (-g["count"], STAGE_RANK[g["stage"]], g["message"]))


def render_md(report: dict, worst: int) -> tuple[str, str]:
    s = report["summary"]
    m = report["meta"]
    L = [f"# {TOOL} corpus round trip", ""]
    L.append(f"Commit `{m.get('commit') or '?'}` · {s['processed']} of {s['files']} files · timeout {m['timeout']} s per step · "
             f"LibreOffice comparison: {'on' if m['soffice'] else 'off'}")
    if s["not_run_deadline"]:
        L.append(f"\n**{s['not_run_deadline']} files were not run: the time budget ran out.**")
    L += ["", "| Stage | ok | err | panic | crash | timeout | skipped |", "|---|---|---|---|---|---|---|"]
    for stage in STAGES + ("check",):
        c = s[stage] if stage != "check" else s["check"]["status"]
        L.append(f"| {stage} | {c.get('ok', 0)} ({pct(c.get('ok', 0), s['processed'])}) | {c.get('err', 0)} | {c.get('panic', 0)} | "
                 f"{c.get('crash', 0)} | {c.get('timeout', 0)} | {c.get('skipped', 0)} |")
    L += ["", f"Full round trip (open, save, reopen, valid package): **{s['full_round_trip_ok']}** "
          f"({pct(s['full_round_trip_ok'], s['processed'])}); verdict ok (content kept ≥ 99%, formulas unchanged): "
          f"**{s['verdicts'].get('ok', 0)}**.", ""]
    L += ["| Content kept | files | identical | ≥ 0.99 | < 0.90 | median | mean |", "|---|---|---|---|---|---|---|"]
    labels = {"values_kept": "app cell values, original vs saved", "xml_cells_kept": "worksheet XML cells with a value",
              "xml_formulas_kept": "worksheet XML formula cells", "lo_cells_kept": "LibreOffice CSV cells, original vs saved",
              "open_cells_vs_xml": "app cells vs worksheet XML cells (open only)"}
    for key, label in labels.items():
        if key in s:
            v = s[key]
            L.append(f"| {label} | {v['files']} | {v['identical']} | {v['ge_0.99']} | {v['lt_0.90']} | {v['median']} | {v['mean']} |")
    v = s["cell_values"]
    L += ["", f"Cell values compared in {v['files_compared']} files: {v['cells_before']} → {v['cells_after']} cells, "
          f"{v['value_diffs']} differ ({v['files_with_value_diffs']} files); formulas {v['formulas_before']} → {v['formulas_after']}, "
          f"{v['formula_diffs']} differ ({v['files_with_formula_diffs']} files)."]
    c = s["check"]
    L += ["", f"`gridcraft-cli check` (calc oracle + round trip) on {c['files']} files: {c['checked']} formulas checked against cached "
          f"values, {c['matched']} matched, {c['mismatched']} mismatched ({c['files_with_calc_mismatch']} files), {c['no_oracle']} without a "
          f"cached value; round trip clean in {c['round_trip_clean']} files, failed in {c['round_trip_errors']}."]
    if c["categories_differing"]:
        L.append("Round-trip categories differing (files): " + ", ".join(f"{k}: {n}" for k, n in c["categories_differing"].items()))
    if c["top_signatures"]:
        L.append("Top calc mismatch signatures: " + ", ".join(f"`{k}` {n}" for k, n in c["top_signatures"].items()))
    if s["structure_changes"]:
        L += ["", "document.inspect counts that changed on the round trip (files): " + ", ".join(f"{k}: {n}" for k, n in s["structure_changes"].items())]
    L += ["", "| Source | files | open ok | round trip ok |", "|---|---|---|---|"]
    for src, v in s["by_source"].items():
        L.append(f"| {src} | {v['files']} | {v['open_ok']} | {v['round_trip_ok']} |")
    L += ["", "## Top errors", "", "| # | stage | status | message | example |", "|---|---|---|---|---|"]
    for g in report["error_groups"][:25]:
        L.append(f"| {g['count']} | {g['stage']} | {g['status']} | {md_cell(g['message'])} | {md_cell(g['files'][0])} |")
    summary = "\n".join(L) + "\n"
    W2 = ["", f"## Worst {worst} files", "", "| verdict | values | XML cells | XML formulas | LO cells | file | detail |", "|---|---|---|---|---|---|---|"]
    for r in report["worst"][:worst]:
        detail = next((r.get(f"{k}_error") for k in STAGES + ("check",) if r.get(f"{k}_error")), "")
        if not detail and r.get("examples"):
            detail = "; ".join(r["examples"][:2])
        if not detail and r.get("count_changes"):
            detail = ", ".join(f"{k} {a}→{b}" for k, (a, b) in r["count_changes"].items())
        W2.append(f"| {r['verdict']} | {r.get('values_kept', '')} | {r.get('xml_cells_kept', '')} | {r.get('xml_formulas_kept', '')} | "
                  f"{r.get('lo_cells_kept', '')} | [{md_cell(r['file'])}]({r['url']}) | {md_cell(detail)} |")
    return summary, summary + "\n".join(W2) + "\n"


def git_commit() -> str | None:
    if os.environ.get("GITHUB_SHA"):
        return os.environ["GITHUB_SHA"][:12]
    try:
        return subprocess.run(["git", "rev-parse", "--short=12", "HEAD"], capture_output=True, text=True, timeout=10).stdout.strip() or None
    except (OSError, subprocess.SubprocessError):
        return None


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--bin", required=True, help=f"path to the release-built {TOOL}")
    ap.add_argument("--corpus", required=True, help="directory written by fetch.sh (with manifest.tsv)")
    ap.add_argument("--out", required=True, help="directory for corpus-report.json/.md")
    ap.add_argument("--timeout", type=float, default=30.0, help="seconds per step (default 30)")
    ap.add_argument("--jobs", type=int, default=os.cpu_count() or 2)
    ap.add_argument("--soffice", nargs="?", const="soffice", default=None, help="also compare LibreOffice's CSV of every sheet (soffice on PATH or a path)")
    ap.add_argument("--no-check", action="store_true", help=f"skip `{TOOL} check` (calc oracle + round trip)")
    ap.add_argument("--deadline", type=float, default=0, help="Unix time after which no new file is started")
    ap.add_argument("--limit", type=int, default=0, help="only the first N files (0 = all)")
    ap.add_argument("--worst", type=int, default=40, help="files listed in the worst table")
    ap.add_argument("--keep", action="store_true", help="keep the saved round-trip files in OUT/work")
    args = ap.parse_args()
    args.bin = os.path.abspath(args.bin)
    args.corpus = os.path.abspath(args.corpus)
    if args.soffice:
        found = shutil.which(args.soffice) or shutil.which("libreoffice")
        if not found:
            print(f"soffice not found ({args.soffice}); LibreOffice comparison off", file=sys.stderr)
        args.soffice = found
    os.makedirs(args.out, exist_ok=True)
    workdir = os.path.join(os.path.abspath(args.out), "work")
    os.makedirs(workdir, exist_ok=True)

    with open(os.path.join(args.corpus, "manifest.tsv"), encoding="utf-8") as f:
        header = f.readline().rstrip("\n").split("\t")
        entries = [dict(zip(header, line.rstrip("\n").split("\t"))) for line in f if line.strip()]
    for i, e in enumerate(entries):
        e["n"] = i
    if args.limit:
        entries = entries[: args.limit]

    env = dict(os.environ, RUST_BACKTRACE="0", LC_ALL="C.UTF-8")
    recs: list[dict] = []
    not_run = 0
    lock = threading.Lock()
    t0 = time.monotonic()

    def job(e: dict):
        if args.deadline and time.time() > args.deadline:
            return None
        try:
            return process(e, args, workdir, env)
        except Exception as ex:  # a bug here must not end the run
            return {"file": e["file"], "source": e["source"], "url": e["url"], "bytes": int(e["bytes"]), "verdict": "runner:error",
                    "runner_error": f"{type(ex).__name__}: {ex}"}

    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, args.jobs)) as pool:
        for i, rec in enumerate(pool.map(job, entries), 1):
            with lock:
                if rec is None:
                    not_run += 1
                    continue
                recs.append(rec)
                if i % 25 == 0 or ":" in rec["verdict"]:
                    print(f"[{i}/{len(entries)} {time.monotonic() - t0:.0f}s] {rec['verdict']:<16} {rec['file']}", flush=True)

    recs.sort(key=lambda r: r["file"])
    report = {
        "meta": {"tool": TOOL, "bin": args.bin, "commit": git_commit(), "generated": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                 "timeout": args.timeout, "jobs": args.jobs, "soffice": bool(args.soffice), "wall_seconds": round(time.monotonic() - t0, 1)},
        "summary": summarize(recs, not_run),
        "error_groups": error_groups(recs),
        "worst": [],
        "files": recs,
    }
    keys = ("file", "url", "verdict") + FIDELITY_KEYS + tuple(f"{s}_error" for s in STAGES + ("check",)) + ("count_changes",)
    report["worst"] = [
        {k: v for k, v in dict(r, examples=(r.get("values_cmp") or {}).get("examples")).items() if k in keys + ("examples",) and v not in (None, {}, [])}
        for r in sorted(recs, key=badness)
        if r["verdict"] != "ok"
    ][: args.worst]
    with open(os.path.join(args.out, "corpus-report.json"), "w", encoding="utf-8") as f:
        json.dump(report, f, indent=1, ensure_ascii=False)
    summary_md, full_md = render_md(report, args.worst)
    with open(os.path.join(args.out, "corpus-report.md"), "w", encoding="utf-8") as f:
        f.write(full_md)
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a", encoding="utf-8") as f:
            f.write(full_md if len(full_md) < 900_000 else summary_md)
    print(summary_md)
    if not args.keep:
        shutil.rmtree(workdir, ignore_errors=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
