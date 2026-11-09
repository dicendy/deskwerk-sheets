# Chart parity

> **Last reviewed:** 2026-10-18 · **Last updated:** 2026-11-08 · **Change:** minor (kept XLSX charts: a new title, legend position or type is patched into the original chart XML, keeping its fonts and series formatting; surface charts are kept byte for byte; pie/doughnut percentage labels; logarithmic value axis; secondary line/area/column groups keep their kind; earlier: value axis: fixed minimum/maximum, major unit and an own number format are read, drawn and written; charts opened from XLSX keep their original chart XML and its style, colour and shape parts on save, following moved series references and refreshing value caches; earlier: chart-data-table keys pass the series index to the shared combo plot-kind rule, restoring compilation after #1150; earlier: doughnut charts: every series is a concentric ring, the first series innermost; hole size and first slice angle kept; rotated slices cover their own sweep; legend names categories; earlier: charts resize from eight handles, chart data tables, legend keys and borders, native XLSX round-trip and print/PDF; earlier: selected chart source outlines stay beneath floating objects) · **Target:** Microsoft Excel (Microsoft 365), Excel for Mac 16.113.4

Excel's chart types and chart elements against GridCraft. Part of
[target-app-parity.md](target-app-parity.md). **Breadth ~50%, ready ~30%, 35–55 h** (estimated).

Sources: `crates/model/src/features.rs` (`ChartKind`, `Chart`, `Series`), `crates/chart/src`
(layout and rendering), `crates/xlsx/src/chart.rs` (DrawingML chart parts), merged PRs #62, #116,
#158 and #161, and issue #190. Behaviour checked against Microsoft's
[Pareto](https://support.microsoft.com/en-us/excel/create-a-pareto-chart) and
[pie/doughnut](https://support.microsoft.com/en-us/excel/add-a-pie-chart) support pages.

## Chart types

| Excel type | Excel subtypes | GridCraft | Notes |
|---|---|---|---|
| Column | clustered, stacked, 100% stacked, 3-D clustered/stacked/100%, 3-D column | 3 of 7 | no 3-D |
| Bar | clustered, stacked, 100%, 3-D ×3 | 3 of 6 | no 3-D |
| Line | line, stacked, 100% stacked, with markers ×3, 3-D line | 3 of 7 (line, with markers, stacked) | no 100% stacked; smoothed lines landed (#62) |
| Pie | pie, 3-D pie, pie of pie, bar of pie, doughnut | 2 of 5 (pie, doughnut) | no exploded, pie of pie, bar of pie; doughnut draws every series as a concentric ring, first series innermost; its legend names categories, matching Excel's default vary-by-category colours |
| Area | area, stacked, 100%, 3-D ×3 | 3 of 6 | no 3-D; 100% stacked drawn as shares of each category's total and kept as `percentStacked` in XLSX |
| X Y (scatter) | markers, smooth lines (±markers), straight lines (±markers) | 2 of 5 | no smooth lines |
| Bubble | bubble, 3-D bubble | 1 of 2 | |
| Stock | high-low-close, open-high-low-close, volume-HLC, volume-OHLC | 1 kind | |
| Surface | 3-D surface, wireframe, contour, wireframe contour | 0 of 4 | read from XLSX as a line chart over the same data and written back byte for byte; a new title or legend is patched into the kept part, a new type writes the chart from the model |
| Radar | radar, with markers, filled | 1 of 3 | |
| Treemap, Sunburst | 1 each | 2 of 2 | |
| Histogram, Pareto | 2 | 1 of 2 | no Pareto |
| Box & Whisker | 1 | 1 of 1 | exclusive (default) and inclusive quartiles per series; XLSX round-trip; `chart.set {quartiles}` changes all series |
| Waterfall, Funnel | 2 | 2 of 2 | lose their kind in XLSX (PR #161) |
| Box & Whisker | 1 | 1 of 1 | |
| Waterfall, Funnel | 2 | 2 of 2 | |
| Filled map | 1 | 0 | needs geographic boundary data (licensing decision) |
| Combo | custom combinations, secondary axis | yes (line on secondary axis) | |
| **Total** | ~60 subtypes | **26 (~43%)** | usage-weighted breadth higher (~65%): column, bar, line, pie and scatter dominate |

## Chart elements and formatting

| Element | Excel | GridCraft |
|---|---|---|
| Move and resize the chart object | yes | yes (eight handles, each moving only its own edges; Format Chart Area Size fields) (#190) |
| Selected chart source-data outline | visible on uncovered cells, beneath floating objects | painted above cells and below charts, shapes, pictures and slicers; retains pane clipping and right-to-left cell geometry (#2861) |
| Chart title, axis titles | text, linked to cells, full formatting | text only |
| Legend | position, overlay, formatting, per-entry delete | position only |
| Data labels | value, category, series, percentage, from cells, callouts, position | value on/off; pie and doughnut slices can show their percentage of the total, alone or after the value (`showPercent`, read, drawn and written; Format Chart pane Elements ▸ Show percentage, `chart.set {dataLabelPercent}`) |
| Gridlines | major/minor per axis, formatting | one on/off flag |
| Axis options | min/max/units, log scale, reverse order, crossing point, display units, date axis, number format | primary value axis: minimum, maximum, major unit, a number format not linked to the source and a logarithmic scale (`c:logBase`, base 2–1000), read, drawn (values past a fixed bound are cut at the plot edge) and written; set in the Format Chart pane (Value Axis ▸ Logarithmic scale, base 10 by default, and its Base field) or with `chart.set {logBase: 2..1000 | null}`; bounds, unit and number format have no UI yet. On a log axis zero and negative points are left out, columns rise from the bottom of the plot and the major unit is one power; scatter and bubble charts and the secondary axis stay linear. Category axis, reverse order, crossing point, display units: **missing** |
| Multi-level category axis (a category range two or more columns wide beside the values: year beside month) | the outer levels grouped under the labels with separators | read (`multiLvlStrRef`), drawn (rows under the labels; columns beside them in bar charts; up to three outer levels), written back with Excel's level caches |
| Doughnut hole size, first slice angle | 10–90% hole (Excel default 75%), 0–359° first slice | Format Chart pane sliders; read from and written to `c:holeSize` / `c:firstSliceAng`; documents saved before the setting existed keep their old 50% hole |
| Secondary axis | any series | combo charts: line, area or column series on their own automatic axis on the right, read, drawn and written with their group's kind; the primary axis options are read from the first group's axis wherever it is listed. No options of its own |
| Trendlines | linear, exponential, logarithmic, polynomial, power, moving average, equation and R² | all six types, several per series, name, forward/backward forecast, set intercept, equation and R² label, line colour/width/dash; read, written and drawn on 2-D column, bar, line, area, scatter and bubble series; `chart.addElement` `trendline` adds/removes them. No Format Trendline pane, label position/number format not kept |
| Error bars | fixed, %, standard deviation/error, custom | **missing** |
| Up/down bars, high-low lines, drop lines, series lines | yes | **missing** |
| Data table under the chart | yes | yes for column, bar, line, area and compatible combo charts: live formatted values, blank cells, grouped category headers, optional legend keys, horizontal/vertical borders and outline; native `c:dTable` XLSX round-trip; shared UI/print/PDF rendering |
| Per-series / per-point fill, line, marker, effects | Format pane for every element | series colour only; Format Chart pane covers chart-level options |
| Chart styles and colour palettes | ~16 styles × colour sets | style number stored, own palettes |
| Chart sheets | yes | **kept, not editable**: not shown, and written back unchanged in their place on save (#727); left out when a sheet they plot is deleted or renamed. No F11 / Move Chart > New sheet |
| Chart templates (`.crtx`) | yes | **missing** |
| Recommended Charts | yes | dialog with recommendations from the data's shape (own heuristic) and an All Charts list, live previews (#215) |
| Switch Row/Column, Select Data | yes | yes (Switch on charts read from files landed in #116) |
| Recommended Charts | yes | partial (Insert ▸ Recommended) |
| Switch Row/Column, Select Data | yes | yes (Switch on charts read from files landed in #116); Select Data dialog: data range, remove series, Horizontal (Category) Axis Labels (#287). No series add/edit or hidden/empty cell options |
| Sparklines (line, column, win/loss, markers, axis) | yes | line, column, win/loss |
| PivotCharts | yes | Insert ▸ PivotChart (`insert.pivotChart`): bound to a PivotTable's report, rebuilt on refresh and layout changes, `c:pivotSource` round trip; no field buttons, innermost category level only |
| Animation on data change | yes (Windows) | no |

Data tables are controlled by `chart.addElement {element: "dataTable", on: true,
 dataTableKeys: true}` or `chart.set {dataTable: true}`. `chart.set` also accepts
`dataTableKeys`, `dataTableHorizontalBorder`, `dataTableVerticalBorder` and `dataTableOutline`.
Changing to an unsupported chart kind removes the table (Undo restores it). The chart legend
is independent. Bar charts keep their vertical category axis; other supported charts use the
table headers instead of repeating category-axis labels. Individual labels and values that
exceed a cell's width are shortened with an ellipsis. Tables are bounded to 128 categories,
32 series and half the available chart height; if they cannot fit, a “Data table needs more
space” notice replaces the table. Enlarge the chart or reduce its source range. Table-specific
font/fill styling and exact Excel layout fidelity are not implemented; Excel-side verification
has not been performed.

Format reference: [Microsoft's DrawingML data table documentation](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.drawing.charts.datatable?view=openxml-3.0.1).

## XLSX round-trip

Charts are read from and written to DrawingML chart parts. An embedded chart read from a file
keeps its original chart part, with the chart style, colour and user-shape parts behind it, and
that part is written back in place of GridCraft's own while the model's chart is unchanged but
for its position, series references, title, legend position and type (#1668). So axis scaling,
label options, error bars, per-point formats, fonts, layout and extensions survive open → edit
cells → save:

- Series references that moved (rows or columns inserted or deleted, a cut and paste, a sheet
  renamed) are rewritten in the kept `<c:f>` elements (#1676), and the number and text caches of
  the series are rebuilt from the cells when they no longer match (#1677). Nothing else in the
  part changes; an untouched chart is written back byte for byte.
- A new title, legend position or type is patched into the kept XML too (#1675). The title keeps
  the paragraph and run properties of the old one (or its text properties when it had no rich
  text) and its place; removing it sets `autoTitleDeleted`, and a title taken from a cell is replaced by the
  typed text. A moved legend keeps its font and loses its manual layout; a removed legend is
  deleted. A type change within column, bar, line and area (clustered, stacked and 100% stacked)
  rewrites the one plot group in place, keeping each series' fill, line, markers, labels and
  trendlines: a fill becomes the line colour of a line series and back, the axes swap sides
  between columns and bars, and label positions the new type doesn't allow are dropped. The
  patched part is read back and must match the model, or the chart is written from the model.
- The chart is written from the model instead (losing what the model doesn't hold) when it was
  edited otherwise (colours, axis options, data labels, a series added or removed, a series
  switched between a reference and a literal), when the type changes to or from another family
  (pie, scatter, radar, stock and others), or between charts with more than one plot group
  (combos, a secondary axis), when the kept XML doesn't parse, or when a formula outside the series
  (a title or data label taken from a cell, error bars) names a sheet whose series references
  moved, since such a formula isn't in the model and can't follow. A formula outside the series on
  another sheet is kept as it was, not adjusted.
- Surface charts (`surfaceChart`, `surface3DChart`) are read as line charts over the same series,
  so they are shown and kept: untouched they are written back byte for byte, with a new title or
  legend they are patched in place, and with a new type they are written from the model (#2403).
- Charts created in GridCraft and chartex charts (treemap, sunburst, histogram, box, waterfall,
  funnel) are written from the model.

What the model doesn't hold is still not shown on screen: label options other than values and
slice percentages, per-point formats, and axis options other than the primary value axis's
bounds, major unit, number format and log scale are kept in the file but drawn with GridCraft's
defaults. A surface chart is drawn as lines.

Series data, categories and names may refer to defined names, as dynamic (OFFSET) ranges do:
Excel writes a workbook-scoped name as `[0]!Name` in `<c:f>`. GridCraft reads it as the bare
name and writes it back with `[0]!`; a sheet-scoped name is written `Sheet!Name`. Excel needs the
qualifier: with a bare name in `<c:f>` it draws the whole chart blank, without a repair prompt
(observed in Excel for Windows, Microsoft 365). External books (`[1]Sheet1!…`) are kept as they
are (#1686). Such series get no `numCache`/`strCache` on save (#1690).

## Work (35–55 h)

1. Kept chart XML: patch other edits (colours, axis options, data labels, combo types) in place instead of writing the chart from the model; adjust formulas outside the series (titles from cells, error bars). Title, legend and type patches landed (#1675): 2–4 h.
2. Axis options beyond the value axis's bounds, unit, format and log scale (reverse order, crossing, display units, secondary-axis options, category axes, log scale on scatter), a Format Axis pane, label options beyond values and percentages, gridlines per axis, legend formatting: 6–10 h.
3. Error bars; Format Trendline pane (trendlines landed): 4–6 h.
4. Per-element Format pane parity, smoothed lines, exploded pie, pie/bar of pie, 100% variants: 8–12 h.
5. 3-D column/bar/pie/area, surface and contour, Pareto: 8–12 h.
6. Chart sheets, templates and PivotChart field buttons: 5–8 h. Filled map: owner decision on boundary data.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-11-08 | minor | Kept XLSX charts with a new title, legend position or type within column/bar/line/area are patched in place, keeping fonts and series formatting, and read back to check (#1675); other edits still write from the model. Surface charts are read as lines and kept byte for byte instead of dropped (#2403). Pie/doughnut percentage labels and a logarithmic primary value axis are read, drawn and written; secondary line, area and column groups keep their kind on save and the primary axis is found by id. Round-trip item 3–5 h → 2–4 h, axis item 8–12 h → 6–10 h, total 35–55 h. |
| 2026-11-08 | minor | Embedded charts read from XLSX keep their chart part and style, colour and user-shape parts on save while unedited (#1668, based on #2404); moved series references are rewritten in the kept XML and stale caches rebuilt (#1676, #1677), so inserting rows or renaming a sheet no longer loses chart formatting. Round-trip work item shrinks from 6–10 h to 3–5 h. Value axis minimum, maximum, major unit and own number format are read, drawn (out-of-range values clipped) and written; axis work item shrinks to 8–12 h. |
| 2026-11-07 | minor | Chart-data-table keys use the shared plot-kind helper with their series index after #1150. A focused regression checks the first combo series uses a column key and later series use line keys. No readiness score change. |
| 2026-11-02 | minor | Charts move and resize from eight handles, each moving only its own edges (#190); resizing leaves the work list (1–2 h, within the 40–60 h rounding) |
| 2026-11-01 | minor | Chart data tables for column, bar, line, area and compatible combo charts: command/UI controls, formatted source values, blanks, grouped category headers, legend keys, independent borders, shared print/PDF rendering and native c:dTable import/export. Dense tables show a space notice; no readiness score change. |
| 2026-10-29 | minor | Selected charts highlight their source cells beneath floating objects, so the blue outline no longer crosses the chart; uncovered source cells remain highlighted (#2861) |
| 2026-10-24 | minor | 100% Stacked Area: inserted from the ribbon and All Charts, drawn as shares of each category's total, read and written as `percentStacked` |
| 2026-10-18 | trivial | Stale notes: stock/combo, treemap/sunburst, box and waterfall kinds survive XLSX round-trip (PRs #158, #161) |
| 2026-10-18 | minor | Doughnut charts: every series is a ring (first series innermost), hole size and first slice angle modelled, editable and round-tripped; rotated slices split at 12 o'clock without overlap; legend names categories like Excel's default (#1823) |
| 2026-10-16 | trivial | Series that refer to defined names (`[0]!Name`, `Sheet!Name`) resolve instead of showing `#NAME?`, and are saved qualified, as Excel needs (#1686) |
| 2026-10-16 | minor | Box & whisker quartiles default to exclusive, honor each series on draw and XLSX round-trip; `chart.set {quartiles}` changes the method (#1453) |
| 2026-10-13 | minor | Multi-level category axes: read, drawn (outer levels under or beside the labels), written back with level caches |
| 2026-10-12 | trivial | chart sheets are kept on save, still not editable (#727) |
| 2026-10-11 | minor | Trendlines: all six types modelled, round-tripped through `c:trendline`, drawn with forecast and equation/R² labels; `chart.addElement` `trendline` |
| 2026-10-11 | minor | PivotCharts (`insert.pivotChart`) |
| 2026-10-11 | minor | Recommended Charts dialog with previews (#215) |
| 2026-10-11 | minor | Select Data dialog with category axis labels; a blank top-left cell makes the first column the categories (#287) |
| 2026-10-10 | minor | Smooth lines (#62) and Switch Row/Column on file charts (#116) landed; chart resize bug (#190) |
| 2026-10-10 | major | Created from a source audit: 25 kinds (25 of ~60 subtypes), element gaps, round-trip loss |
