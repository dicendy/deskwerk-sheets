//! Deskwerk Sheets' icon set, drawn in code on a 16×16 design grid (original artwork; see
//! ATTRIBUTION.md). Large ribbon icons use the same drawings at 32×32 with accent colours.
//! Accent and paper colours come from the theme tokens, so icons follow light and dark mode;
//! icons placed on the sheet ([`paint_on_sheet`]) keep the light colours, as the sheet does.

use egui::epaint::{CubicBezierShape, PathShape, PathStroke};
use egui::{Color32, Painter, Pos2, Rect, Stroke, pos2, vec2};

use crate::theme::{Cat, Tokens};

/// The glyph on a Function Library book ([`Icon::FnBook`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FnGlyph {
    /// Recently Used: a star.
    Star,
    /// Financial: a stack of coins.
    Coins,
    /// Logical: a question mark.
    Question,
    /// Text: a letter.
    Letter,
    /// Date & Time: a clock face.
    Clock,
    /// Lookup & Reference: a magnifier.
    Magnifier,
    /// Math & Trig: theta.
    Theta,
    /// More Functions: three dots.
    Dots,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    Paste,
    Cut,
    Copy,
    Brush,
    Bold,
    Italic,
    Underline,
    Strike,
    Borders,
    BorderAll,
    BorderNone,
    BorderOutside,
    BorderThick,
    Fill,
    FontColor,
    AlignTop,
    AlignMiddle,
    AlignBottom,
    AlignLeft,
    AlignCenter,
    AlignRight,
    Orientation,
    IndentDec,
    IndentInc,
    Wrap,
    Merge,
    Currency,
    /// The Accounting Number Format button in a German or French number locale.
    CurrencyEuro,
    Percent,
    Comma,
    DecInc,
    DecDec,
    CondFormat,
    FormatTable,
    CellStyles,
    Insert,
    Delete,
    Format,
    Sum,
    FillDown,
    Eraser,
    SortFilter,
    Find,
    Undo,
    Redo,
    Save,
    Home,
    More,
    Chevron,
    ChevronUp,
    Close,
    Check,
    Fx,
    Plus,
    Left,
    Right,
    Normal,
    PageLayout,
    PageBreak,
    Minus,
    Table,
    PivotTable,
    Chart,
    ChartBar,
    ChartLine,
    ChartPie,
    ChartScatter,
    Picture,
    Shapes,
    Link,
    Comment,
    Note,
    TextBox,
    Symbol,
    /// An equation: a radical sign over an italic x.
    Equation,
    Sparkline,
    Checkbox,
    /// A form button with a pointer.
    Button,
    /// A form slider (scroll bar).
    Slider,
    /// A form spin button.
    Spin,
    Function,
    Name,
    Trace,
    Calc,
    Filter,
    SortAsc,
    SortDesc,
    Duplicates,
    TextColumns,
    Validation,
    Group,
    Ungroup,
    Subtotal,
    Lock,
    Spell,
    Freeze,
    Zoom,
    Gridlines,
    Theme,
    Margins,
    Orient,
    SheetRtl,
    Paper,
    PrintArea,
    Script,
    Pen,
    Share,
    Search,
    Settings,
    /// Light/dark appearance: a circle, its right half filled.
    Appearance,
    Sheet,
    /// All Sheets navigator next to the sheet tabs: a list of rows with a bullet each.
    SheetList,
    Book,
    Folder,
    /// A book with a coloured spine and a category glyph (Formulas › Function Library). The
    /// spine and glyph take the accent colour of [`paint_accent`].
    FnBook(FnGlyph),
    /// Header & Footer: a page with its top / bottom band marked.
    Header,
    Footer,
    PageNumber,
    PageCount,
    Calendar,
    Clock,
    FileName,
    /// File › Backstage navigation (line art, drawn in the navigation colour).
    Back,
    NewFile,
    OpenFolder,
    Info,
    Print,
    /// A macro: a script page with a run arrow.
    Macro,
    /// Record Macro: a ring around the recording dot.
    Record,
    /// Use Relative References: a cell and an arrow to a cell offset from it.
    Relative,
    /// A reference box's button: shrink the window to the box (Collapse) or restore it (Expand).
    Collapse,
    Expand,
    /// The macro debugger's Run: a play triangle.
    Play,
    /// Step: an arrow down onto a line.
    Step,
    /// Continue: a bar and a play triangle.
    Continue,
    /// Run to Here: an arrow to a marked line.
    RunTo,
    /// Stop: a square.
    StopSquare,
    /// Increase / Decrease Font Size: an A with a caret up / down.
    FontGrow,
    FontShrink,
    /// Show / Hide Detail: rows beside a box with a plus / minus.
    ShowDetail,
    HideDetail,
    /// Delete Comment: a speech bubble with a red cross.
    DeleteComment,
    /// Delete Chart: a column chart with a red cross.
    DeleteChart,
    /// Zoom to 100%: a magnifier with 100 in its lens.
    Zoom100,
    /// Zoom to Selection: a magnifier over a dashed selection.
    ZoomSelection,
    /// Watch Window: a window with a pair of glasses.
    WatchWindow,
    /// Data › Sort (the Sort dialog): sort levels in a box and a down arrow.
    SortCustom,
    /// Advanced Filter: a funnel and a criteria range.
    FilterAdvanced,
    /// Slicer: a panel of stacked filter buttons.
    Slicer,
    /// Clear Filter: a funnel with a red cross.
    ClearFilter,
    /// Reapply: a funnel with a circular arrow.
    Reapply,
    /// Draw › Eraser: an eraser over an ink stroke.
    InkEraser,
    /// Switch Row/Column: a grid with a bent two-headed arrow.
    SwitchRowCol,
    /// Check Accessibility: a person with open arms.
    Accessibility,
    /// Allow Edit Ranges: a grid and a pencil.
    AllowEdit,
    /// Error Checking: a warning triangle.
    ErrorCheck,
    /// Workbook Statistics: a page with 123.
    Statistics,
    /// What-If Analysis: a grid and a question mark.
    WhatIf,
    /// Evaluate Formula: fx under a magnifier.
    EvaluateFormula,
    /// Calculate Now: a calculator with a circular arrow.
    CalcNow,
    /// Calculate Sheet: a sheet with a circular arrow.
    CalcSheet,
    /// Print Titles: two pages repeating the same top row.
    PrintTitles,
    /// Resize Table: a table with a corner arrow.
    ResizeTable,
    /// Convert to Range: a table turning into a plain range.
    ConvertToRange,
    /// Select Data: a grid with a dashed range.
    SelectData,
    /// Create from Selection: a range whose top row becomes a name tag.
    CreateFromSelection,
    /// Arrange All: four tiled windows.
    ArrangeAll,
    /// Insert › Icons: a star, a check, a person and a house.
    IconsGallery,
    /// Hierarchy charts: a treemap.
    ChartTreemap,
    /// Recommended Charts: columns with a star.
    ChartRecommended,
    /// Statistic charts: a histogram.
    ChartHistogram,
    /// Add Chart Element: columns with a plus.
    AddChartElement,
    /// Change Chart Type: columns turning into a pie.
    ChangeChartType,
    /// Data Analysis: columns under a magnifier.
    DataAnalysis,
    /// Forecast Sheet: a line with a dashed projection.
    Forecast,
    /// Select Objects: a pointer over a dashed frame.
    SelectObjects,
    /// Ink to Shape: a scribble turning into a shape.
    InkToShape,
    /// Black and red pens, and the highlighter (Pen is the blue pen).
    PenBlack,
    PenRed,
    Highlighter,
    /// New Comment: a speech bubble with a plus.
    NewComment,
    /// Show Comments: two speech bubbles.
    ShowComments,
    /// Previous / Next Comment: a speech bubble with an arrow.
    PrevComment,
    NextComment,
    /// Protect Workbook: a workbook with a lock.
    ProtectWorkbook,
    /// Switch Windows: two overlapping windows.
    SwitchWindows,
    /// Show Formulas: a sheet with an equals sign.
    ShowFormulas,
    /// Use in Formula: fx with a name tag.
    UseInFormula,
    /// Solver: a target.
    Solver,
    /// Define Name: a name box with a pencil.
    DefineName,
    /// Trace Precedents: arrows from cells into a cell.
    TracePrecedents,
    /// Trace Dependents: arrows from a cell out to cells.
    TraceDependents,
    /// Remove Arrows: a trace arrow with a red cross.
    RemoveArrows,
    /// Page Break Preview: a page in a thick frame split by a dashed line.
    PageBreakPreview,
    /// A PDF document: a page with a red band.
    Pdf,
    /// New Window: a window with a plus.
    NewWindow,
    /// Command Palette: a prompt box.
    CommandPalette,
    /// Agent Control: a robot head.
    AgentControl,
    /// Action Journal: a ruled page with a clock.
    ActionJournal,
    /// Check for Updates: a circular arrow around a down arrow.
    CheckUpdates,
    /// New Macro: a script page with a plus.
    MacroNew,
    /// Chart Title: a column chart under a title bar.
    ChartTitle,
    /// Format Chart pane: a window with a side pane.
    FormatPane,
    /// Get Data: a text file flowing into a grid.
    GetData,
    /// Flash Fill: a column and a lightning bolt.
    FlashFill,
    /// An account: head and shoulders.
    Person,
}

struct G<'a> {
    p: &'a Painter,
    r: Rect,
    s: f32,
}

impl G<'_> {
    fn pt(&self, x: f32, y: f32) -> Pos2 {
        pos2(self.r.left() + x * self.s, self.r.top() + y * self.s)
    }
    fn line(&self, pts: &[(f32, f32)], c: Color32, w: f32) {
        let v: Vec<Pos2> = pts.iter().map(|(x, y)| self.pt(*x, *y)).collect();
        self.p.add(PathShape::line(v, PathStroke::new(w * self.s.clamp(1.0, 2.0), c)));
    }
    fn closed(&self, pts: &[(f32, f32)], c: Color32, w: f32) {
        let v: Vec<Pos2> = pts.iter().map(|(x, y)| self.pt(*x, *y)).collect();
        self.p.add(PathShape::closed_line(v, PathStroke::new(w * self.s.clamp(1.0, 2.0), c)));
    }
    fn stroke(&self, pts: &[(f32, f32)], c: Color32) {
        self.line(pts, c, 1.15);
    }
    fn rect(&self, x: f32, y: f32, w: f32, h: f32, fill: Option<Color32>, stroke: Option<Color32>) {
        let r = Rect::from_min_size(self.pt(x, y), vec2(w * self.s, h * self.s));
        if let Some(f) = fill {
            self.p.rect_filled(r, 1.0, f);
        }
        if let Some(c) = stroke {
            self.p.rect_stroke(r, 1.0, Stroke::new(1.0, c), egui::StrokeKind::Inside);
        }
    }
    fn poly(&self, pts: &[(f32, f32)], fill: Color32) {
        let v: Vec<Pos2> = pts.iter().map(|(x, y)| self.pt(*x, *y)).collect();
        self.p.add(PathShape::convex_polygon(v, fill, Stroke::NONE));
    }
    fn circle(&self, x: f32, y: f32, r: f32, fill: Option<Color32>, stroke: Option<Color32>) {
        if let Some(f) = fill {
            self.p.circle_filled(self.pt(x, y), r * self.s, f);
        }
        if let Some(c) = stroke {
            self.p.circle_stroke(self.pt(x, y), r * self.s, Stroke::new(1.0, c));
        }
    }
    fn text(&self, x: f32, y: f32, t: &str, size: f32, c: Color32, style: (bool, bool)) {
        let fam = if style.0 {
            crate::theme::CELL_BOLD
        } else if style.1 {
            crate::theme::CELL_ITALIC
        } else {
            crate::theme::CELL
        };
        let font = egui::FontId::new(size * self.s, egui::FontFamily::Name(fam.into()));
        self.p.text(self.pt(x, y), egui::Align2::CENTER_CENTER, t, font, c);
    }
}

/// The colours an icon draws with besides its main colour: the paper inside documents and cells,
/// the category palette for its coloured parts, and the accent a button asked for.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Palette {
    paper: Color32,
    cat: [Color32; 8],
    recording: Color32,
    /// The accent colour: the coloured part of an icon that has one (a book's spine and glyph),
    /// otherwise the icon's main colour. `None` draws the icon as it is.
    accent: Option<Color32>,
}

impl Palette {
    fn of(t: &Tokens) -> Palette {
        Palette { paper: t.paper, cat: t.cat, recording: t.recording, accent: None }
    }

    fn cat(&self, c: Cat) -> Color32 {
        self.cat.get(c as usize).copied().unwrap_or(Color32::GRAY)
    }

    /// A pale wash of a category colour over the paper (cell fills inside icons).
    fn wash(&self, c: Cat) -> Color32 {
        self.cat(c).gamma_multiply(0.3)
    }
}

/// The code-drawn icon that stands in for a character the current fonts cannot draw (`✕` and
/// `✓` have no bundled egui face). Interface chrome uses this instead of painting a tofu box; a
/// workbook cell still needs a real glyph because its characters are user data.
pub fn fallback_for_symbol(c: char) -> Option<Icon> {
    match c {
        '✕' => Some(Icon::Close),
        '✓' => Some(Icon::Check),
        _ => None,
    }
}

/// Whether an icon has a part of its own for the accent colour (otherwise the accent recolours
/// the whole icon).
fn has_accent_part(icon: Icon) -> bool {
    matches!(icon, Icon::FnBook(_))
}

/// Draws `icon` into `rect` (square-ish), in colour `c`, with the current theme's palette.
pub fn paint(p: &Painter, rect: Rect, icon: Icon, c: Color32) {
    paint_with(p, rect, icon, c, &Palette::of(&Tokens::get(p.ctx())));
}

/// Draws `icon` like [`paint`], with an accent colour: the icon's coloured part when it has one
/// (the spine and glyph of [`Icon::FnBook`]), otherwise the whole icon. `None` is [`paint`].
pub fn paint_accent(p: &Painter, rect: Rect, icon: Icon, c: Color32, accent: Option<Color32>) {
    paint_with(p, rect, icon, c, &Palette { accent, ..Palette::of(&Tokens::get(p.ctx())) });
}

/// Draws `icon` on the sheet, which stays white in both themes, so with the light palette.
pub fn paint_on_sheet(p: &Painter, rect: Rect, icon: Icon, c: Color32) {
    paint_with(p, rect, icon, c, &Palette::of(&Tokens::light()));
}

/// Draws `icon` into `rect` (square-ish), in colour `c`, with palette `pal`.
#[allow(clippy::cognitive_complexity)] // One flat arm per icon: long, not tangled.
fn paint_with(p: &Painter, rect: Rect, icon: Icon, c: Color32, pal: &Palette) {
    let side = rect.width().min(rect.height());
    let r = Rect::from_center_size(rect.center(), vec2(side, side));
    let g = G { p, r, s: side / 16.0 };
    let c = match pal.accent {
        Some(a) if !has_accent_part(icon) => a,
        _ => c,
    };
    let light = c.gamma_multiply(0.45);
    let paper = pal.paper;
    let blue = pal.cat(Cat::Blue);
    let green = pal.cat(Cat::Green);
    let red = pal.cat(Cat::Red);
    let orange = pal.cat(Cat::Orange);
    let purple = pal.cat(Cat::Purple);
    let yellow = pal.cat(Cat::Amber);
    match icon {
        Icon::Paste => {
            g.rect(3.0, 2.5, 9.0, 12.0, Some(yellow), None);
            g.rect(5.0, 1.5, 5.0, 2.5, Some(pal.cat(Cat::Slate)), None);
            g.rect(6.5, 6.0, 7.5, 9.0, Some(paper), Some(c));
            for y in [8.5, 10.5, 12.5] {
                g.stroke(&[(8.0, y), (12.5, y)], light);
            }
        }
        Icon::Cut => {
            g.circle(5.0, 12.0, 2.0, None, Some(c));
            g.circle(11.0, 12.0, 2.0, None, Some(c));
            g.stroke(&[(6.3, 10.6), (11.5, 2.0)], c);
            g.stroke(&[(9.7, 10.6), (4.5, 2.0)], c);
        }
        Icon::Copy => {
            g.rect(2.5, 2.5, 7.5, 9.0, Some(paper), Some(c));
            g.rect(6.0, 5.5, 7.5, 9.0, Some(paper), Some(c));
        }
        Icon::Brush => {
            g.rect(3.0, 2.0, 10.0, 4.0, Some(yellow), Some(c));
            g.stroke(&[(8.0, 6.0), (8.0, 8.5), (5.0, 8.5), (5.0, 10.0)], c);
            g.rect(4.0, 10.0, 2.0, 4.5, Some(c), None);
        }
        Icon::Bold => g.text(8.0, 8.5, "B", 12.0, c, (true, false)),
        Icon::Italic => g.text(8.0, 8.5, "I", 12.0, c, (false, true)),
        Icon::Underline => {
            g.text(8.0, 7.5, "U", 11.0, c, (false, false));
            g.stroke(&[(4.0, 14.0), (12.0, 14.0)], c);
        }
        Icon::Strike => {
            g.text(8.0, 8.0, "ab", 10.0, c, (false, false));
            g.stroke(&[(2.5, 8.5), (13.5, 8.5)], c);
        }
        Icon::Borders | Icon::BorderAll => {
            g.rect(2.5, 2.5, 11.0, 11.0, None, Some(light));
            g.stroke(&[(8.0, 2.5), (8.0, 13.5)], light);
            g.stroke(&[(2.5, 8.0), (13.5, 8.0)], light);
            if icon == Icon::Borders {
                g.line(&[(2.0, 13.5), (14.0, 13.5)], c, 1.6);
            } else {
                g.rect(2.5, 2.5, 11.0, 11.0, None, Some(c));
                g.stroke(&[(8.0, 2.5), (8.0, 13.5)], c);
                g.stroke(&[(2.5, 8.0), (13.5, 8.0)], c);
            }
        }
        Icon::BorderNone => {
            for i in 0..6 {
                let t = 2.5 + i as f32 * 2.2;
                g.circle(t, 2.5, 0.4, Some(light), None);
                g.circle(t, 13.5, 0.4, Some(light), None);
                g.circle(2.5, t, 0.4, Some(light), None);
                g.circle(13.5, t, 0.4, Some(light), None);
            }
        }
        Icon::BorderOutside | Icon::BorderThick => {
            g.stroke(&[(8.0, 2.5), (8.0, 13.5)], light);
            g.stroke(&[(2.5, 8.0), (13.5, 8.0)], light);
            g.line(&[(2.5, 2.5), (13.5, 2.5), (13.5, 13.5), (2.5, 13.5), (2.5, 2.5)], c, if icon == Icon::BorderThick { 2.2 } else { 1.2 });
        }
        Icon::Fill => {
            g.poly(&[(3.0, 7.0), (8.0, 2.0), (13.0, 7.0), (8.0, 12.0)], paper);
            g.line(&[(3.0, 7.0), (8.0, 2.0), (13.0, 7.0), (8.0, 12.0), (3.0, 7.0)], c, 1.1);
            g.circle(13.5, 10.0, 1.2, Some(c), None);
        }
        Icon::FontColor => {
            g.text(8.0, 7.0, "A", 11.0, c, (false, false));
        }
        Icon::AlignTop
        | Icon::AlignMiddle
        | Icon::AlignBottom
        | Icon::AlignLeft
        | Icon::AlignCenter
        | Icon::AlignRight
        | Icon::IndentDec
        | Icon::IndentInc => {
            paint_alignment(&g, icon, c, light, blue);
        }
        Icon::Orientation => {
            g.text(6.0, 9.0, "ab", 8.5, c, (false, false));
            g.stroke(&[(9.0, 13.0), (14.0, 3.0)], blue);
            g.poly(&[(14.5, 2.0), (12.0, 4.0), (14.6, 5.2)], blue);
        }
        Icon::Wrap => {
            g.stroke(&[(2.0, 4.0), (14.0, 4.0)], c);
            g.stroke(&[(2.0, 8.0), (12.0, 8.0)], c);
            g.p.add(CubicBezierShape::from_points_stroke(
                [g.pt(12.0, 8.0), g.pt(15.0, 8.0), g.pt(15.0, 12.0), g.pt(11.0, 12.0)],
                false,
                Color32::TRANSPARENT,
                Stroke::new(1.1, blue),
            ));
            g.poly(&[(9.0, 12.0), (11.5, 10.5), (11.5, 13.5)], blue);
            g.stroke(&[(2.0, 12.0), (7.0, 12.0)], c);
        }
        Icon::Merge => {
            g.rect(1.5, 3.5, 13.0, 9.0, None, Some(c));
            g.stroke(&[(4.0, 8.0), (12.0, 8.0)], blue);
            g.poly(&[(3.0, 8.0), (5.5, 6.5), (5.5, 9.5)], blue);
            g.poly(&[(13.0, 8.0), (10.5, 6.5), (10.5, 9.5)], blue);
        }
        Icon::Currency => g.text(8.0, 8.5, "$", 12.0, c, (false, false)),
        Icon::CurrencyEuro => g.text(8.0, 8.5, "€", 12.0, c, (false, false)),
        Icon::Percent => g.text(8.0, 8.5, "%", 11.0, c, (false, false)),
        Icon::Comma => g.text(8.0, 7.0, ",", 14.0, c, (true, false)),
        Icon::DecInc => {
            g.text(5.0, 5.0, ".0", 6.5, c, (false, false));
            g.text(10.0, 11.0, ".00", 6.5, c, (false, false));
            g.stroke(&[(2.0, 12.0), (5.0, 12.0)], blue);
            g.poly(&[(1.0, 12.0), (2.8, 10.6), (2.8, 13.4)], blue);
        }
        Icon::DecDec => {
            g.text(6.0, 5.0, ".00", 6.5, c, (false, false));
            g.text(11.0, 11.0, ".0", 6.5, c, (false, false));
            g.stroke(&[(2.0, 12.0), (5.5, 12.0)], blue);
            g.poly(&[(7.0, 12.0), (5.2, 10.6), (5.2, 13.4)], blue);
        }
        Icon::CondFormat => {
            g.rect(2.0, 2.0, 12.0, 12.0, Some(paper), Some(light));
            g.rect(3.0, 3.5, 4.0, 2.5, Some(red), None);
            g.rect(3.0, 7.0, 7.0, 2.5, Some(yellow), None);
            g.rect(3.0, 10.5, 10.0, 2.5, Some(blue), None);
        }
        Icon::FormatTable => {
            g.rect(2.0, 2.5, 12.0, 11.0, Some(paper), Some(light));
            g.rect(2.0, 2.5, 12.0, 3.0, Some(blue), None);
            g.rect(2.0, 8.0, 12.0, 2.5, Some(pal.wash(Cat::Blue)), None);
            g.stroke(&[(6.0, 2.5), (6.0, 13.5)], light);
            g.stroke(&[(10.0, 2.5), (10.0, 13.5)], light);
        }
        Icon::CellStyles => {
            g.rect(2.0, 3.0, 12.0, 4.5, Some(pal.wash(Cat::Green)), Some(light));
            g.rect(2.0, 8.5, 12.0, 4.5, Some(pal.wash(Cat::Red)), Some(light));
            g.stroke(&[(4.0, 5.2), (9.0, 5.2)], green);
            g.stroke(&[(4.0, 10.7), (9.0, 10.7)], red);
        }
        Icon::Insert | Icon::Delete | Icon::Format => {
            g.rect(2.0, 3.0, 12.0, 10.0, Some(paper), Some(light));
            g.stroke(&[(2.0, 6.5), (14.0, 6.5)], light);
            g.stroke(&[(2.0, 10.0), (14.0, 10.0)], light);
            g.stroke(&[(6.0, 3.0), (6.0, 13.0)], light);
            match icon {
                Icon::Insert => {
                    g.rect(6.0, 6.5, 8.0, 3.5, Some(pal.wash(Cat::Blue)), Some(blue));
                    g.line(&[(1.0, 8.2), (4.5, 8.2)], green, 1.6);
                    g.line(&[(2.75, 6.5), (2.75, 10.0)], green, 1.6);
                }
                Icon::Delete => {
                    g.line(&[(8.0, 6.0), (13.0, 11.0)], red, 1.6);
                    g.line(&[(13.0, 6.0), (8.0, 11.0)], red, 1.6);
                }
                _ => {
                    g.rect(6.0, 6.5, 8.0, 3.5, Some(pal.wash(Cat::Blue)), None);
                }
            }
        }
        Icon::Sum => g.text(8.0, 8.0, "Σ", 13.0, c, (false, false)),
        Icon::FillDown => {
            g.rect(4.0, 2.0, 8.0, 12.0, Some(paper), Some(light));
            g.stroke(&[(8.0, 4.0), (8.0, 11.0)], blue);
            g.poly(&[(5.5, 9.5), (10.5, 9.5), (8.0, 13.0)], blue);
        }
        Icon::Eraser => {
            g.poly(&[(2.0, 10.0), (8.0, 4.0), (12.0, 8.0), (6.0, 14.0)], red.gamma_multiply(0.55));
            g.poly(&[(8.0, 4.0), (10.5, 1.5), (14.5, 5.5), (12.0, 8.0)], red);
        }
        Icon::SortFilter => {
            g.text(4.5, 4.5, "A", 6.0, c, (false, false));
            g.text(4.5, 11.5, "Z", 6.0, c, (false, false));
            g.stroke(&[(8.0, 3.0), (8.0, 13.0)], blue);
            g.poly(&[(6.5, 11.0), (9.5, 11.0), (8.0, 13.5)], blue);
            g.poly(&[(10.0, 3.0), (15.0, 3.0), (13.0, 7.5), (13.0, 12.0), (12.0, 11.0), (12.0, 7.5)], light);
        }
        Icon::Find => magnifier(&g, 6.5, 6.5, 4.0, c, None),
        Icon::Search => {
            g.rect(2.0, 1.5, 8.5, 11.0, Some(paper), Some(c));
            for y in [4.5, 6.5] {
                g.stroke(&[(4.0, y), (8.5, y)], light);
            }
            magnifier(&g, 10.0, 10.0, 3.0, c, Some(paper));
        }
        Icon::Zoom => {
            magnifier(&g, 6.5, 6.5, 4.6, c, Some(paper));
            g.line(&[(6.5, 4.4), (6.5, 8.6)], blue, 1.3);
            g.line(&[(4.4, 6.5), (8.6, 6.5)], blue, 1.3);
        }
        Icon::Zoom100 => {
            magnifier(&g, 7.0, 7.0, 5.4, c, Some(paper));
            g.text(7.0, 7.1, "100", 4.6, blue, (true, false));
        }
        Icon::ZoomSelection => {
            dashed_rect(&g, 1.5, 1.5, 9.5, 8.0, blue);
            magnifier(&g, 9.0, 9.0, 3.2, c, Some(paper));
        }
        Icon::WatchWindow => {
            window(&g, 1.5, 2.0, 13.0, 11.5, c, light, paper);
            g.circle(5.5, 9.5, 2.0, Some(pal.wash(Cat::Blue)), Some(c));
            g.circle(10.5, 9.5, 2.0, Some(pal.wash(Cat::Blue)), Some(c));
            g.stroke(&[(7.5, 9.2), (8.5, 9.2)], c);
        }
        Icon::Undo | Icon::Redo => {
            let flip = icon == Icon::Redo;
            let x = |v: f32| if flip { 16.0 - v } else { v };
            g.p.add(CubicBezierShape::from_points_stroke(
                [g.pt(x(4.0), 7.0), g.pt(x(9.0), 3.0), g.pt(x(15.0), 6.0), g.pt(x(12.0), 12.5)],
                false,
                Color32::TRANSPARENT,
                Stroke::new(1.2, c),
            ));
            g.poly(&[(x(2.5), 7.5), (x(6.5), 4.5), (x(6.5), 9.5)], c);
        }
        Icon::Save => {
            g.rect(2.5, 2.5, 11.0, 11.0, None, Some(c));
            g.rect(5.0, 2.5, 6.0, 4.0, None, Some(c));
            g.rect(4.5, 9.0, 7.0, 4.5, None, Some(c));
        }
        Icon::Home => {
            g.line(&[(2.0, 8.0), (8.0, 2.5), (14.0, 8.0)], c, 1.2);
            g.line(&[(4.0, 6.5), (4.0, 13.5), (12.0, 13.5), (12.0, 6.5)], c, 1.2);
        }
        Icon::SheetList => {
            for y in [4.0, 8.0, 12.0] {
                g.rect(2.5, y - 1.0, 2.0, 2.0, Some(c), None);
                g.stroke(&[(6.5, y), (13.5, y)], c);
            }
        }
        Icon::More => {
            for x in [4.0, 8.0, 12.0] {
                g.circle(x, 8.0, 1.0, Some(c), None);
            }
        }
        Icon::Chevron => g.line(&[(4.5, 6.5), (8.0, 10.0), (11.5, 6.5)], c, 1.2),
        Icon::ChevronUp => g.line(&[(4.5, 10.0), (8.0, 6.5), (11.5, 10.0)], c, 1.2),
        Icon::Left => g.line(&[(10.0, 4.0), (6.0, 8.0), (10.0, 12.0)], c, 1.2),
        Icon::Right => g.line(&[(6.0, 4.0), (10.0, 8.0), (6.0, 12.0)], c, 1.2),
        Icon::Close => {
            g.line(&[(4.0, 4.0), (12.0, 12.0)], c, 1.3);
            g.line(&[(12.0, 4.0), (4.0, 12.0)], c, 1.3);
        }
        Icon::Check => g.line(&[(3.0, 8.5), (6.5, 12.0), (13.0, 4.5)], c, 1.3),
        Icon::Checkbox => {
            g.rect(2.0, 2.0, 12.0, 12.0, None, Some(c));
            g.line(&[(4.5, 8.0), (7.0, 10.5), (11.5, 5.0)], c, 1.4);
        }
        Icon::Button => {
            let frame = Rect::from_min_max(g.pt(1.5, 3.0), g.pt(14.5, 11.0));
            g.p.rect_filled(frame, 2.0 * g.s, paper);
            g.p.rect_stroke(frame, 2.0 * g.s, Stroke::new(1.0, c), egui::StrokeKind::Inside);
            g.poly(&[(9.0, 7.5), (9.0, 15.0), (13.5, 12.5)], c);
        }
        Icon::Slider => {
            g.line(&[(2.0, 8.0), (14.0, 8.0)], light, 1.6);
            g.line(&[(2.0, 8.0), (7.0, 8.0)], blue, 1.6);
            g.circle(7.0, 8.0, 2.4, Some(paper), Some(c));
        }
        Icon::Spin => {
            g.rect(4.0, 1.5, 8.0, 13.0, Some(paper), Some(c));
            g.stroke(&[(4.0, 8.0), (12.0, 8.0)], c);
            g.poly(&[(8.0, 3.2), (10.5, 6.3), (5.5, 6.3)], c);
            g.poly(&[(5.5, 9.7), (10.5, 9.7), (8.0, 12.8)], c);
        }
        Icon::Fx => g.text(8.0, 8.0, "fx", 10.5, c, (false, true)),
        Icon::Plus => {
            g.line(&[(8.0, 3.0), (8.0, 13.0)], c, 1.3);
            g.line(&[(3.0, 8.0), (13.0, 8.0)], c, 1.3);
        }
        Icon::Minus => g.line(&[(3.0, 8.0), (13.0, 8.0)], c, 1.3),
        Icon::Normal | Icon::Gridlines | Icon::Sheet => {
            g.rect(2.0, 2.5, 12.0, 11.0, None, Some(c));
            g.stroke(&[(6.0, 2.5), (6.0, 13.5)], c);
            g.stroke(&[(10.0, 2.5), (10.0, 13.5)], c);
            g.stroke(&[(2.0, 6.2), (14.0, 6.2)], c);
            g.stroke(&[(2.0, 9.8), (14.0, 9.8)], c);
        }
        Icon::PageLayout => {
            // A page under a top ruler and beside a left ruler.
            g.rect(4.5, 4.5, 10.0, 10.0, Some(paper), Some(c));
            g.rect(4.5, 1.0, 10.0, 2.2, Some(pal.wash(Cat::Blue)), None);
            g.rect(1.0, 4.5, 2.2, 10.0, Some(pal.wash(Cat::Blue)), None);
            for k in 0..4 {
                let t = 6.0 + k as f32 * 2.5;
                g.stroke(&[(t, 1.0), (t, 2.2)], c);
                g.stroke(&[(1.0, t), (2.2, t)], c);
            }
            for y in [7.5, 10.0, 12.5] {
                g.stroke(&[(6.5, y), (12.5, y)], light);
            }
        }
        Icon::Paper => {
            // Paper size: a page with its width and height marked.
            g.rect(5.0, 1.5, 9.0, 10.5, Some(paper), Some(c));
            g.stroke(&[(5.0, 14.2), (14.0, 14.2)], blue);
            g.stroke(&[(5.0, 13.2), (5.0, 15.2)], blue);
            g.stroke(&[(14.0, 13.2), (14.0, 15.2)], blue);
            g.stroke(&[(2.5, 1.5), (2.5, 12.0)], blue);
            g.stroke(&[(1.5, 1.5), (3.5, 1.5)], blue);
            g.stroke(&[(1.5, 12.0), (3.5, 12.0)], blue);
        }
        Icon::PageBreakPreview => {
            g.rect(2.0, 2.0, 12.0, 12.0, Some(paper), None);
            for t in [6.0, 10.0] {
                g.stroke(&[(2.0, t), (14.0, t)], light);
            }
            g.line(&[(2.0, 2.0), (14.0, 2.0), (14.0, 14.0), (2.0, 14.0), (2.0, 2.0)], blue, 1.8);
            let mut y = 2.5;
            while y < 13.5 {
                g.line(&[(8.0, y), (8.0, (y + 1.6).min(13.5))], blue, 1.4);
                y += 2.8;
            }
        }
        Icon::Pdf => {
            g.poly(&[(3.0, 1.5), (10.0, 1.5), (13.0, 4.5), (13.0, 14.5), (3.0, 14.5)], paper);
            g.line(&[(3.0, 1.5), (10.0, 1.5), (13.0, 4.5), (13.0, 14.5), (3.0, 14.5), (3.0, 1.5)], c, 1.15);
            g.rect(1.5, 8.0, 10.5, 4.5, Some(red), None);
            g.text(6.75, 10.3, "PDF", 4.2, Color32::WHITE, (true, false));
        }
        Icon::PageBreak => {
            g.rect(2.5, 2.0, 11.0, 12.0, None, Some(c));
            g.line(&[(2.5, 8.0), (13.5, 8.0)], blue, 1.6);
        }
        Icon::Table => table(&g, 2.0, 2.5, 12.0, 11.0, c, light, paper, blue),
        Icon::PivotTable => {
            // A grid with its row and column headers, and the pivot arrow that turns one into
            // the other.
            g.rect(1.5, 1.5, 13.0, 13.0, Some(paper), Some(c));
            g.rect(6.0, 1.5, 8.5, 3.5, Some(pal.wash(Cat::Green)), None);
            g.rect(1.5, 6.0, 3.5, 8.5, Some(pal.wash(Cat::Green)), None);
            g.stroke(&[(1.5, 5.0), (14.5, 5.0)], c);
            g.stroke(&[(5.0, 1.5), (5.0, 14.5)], c);
            g.p.add(CubicBezierShape::from_points_stroke(
                [g.pt(12.0, 7.0), g.pt(12.0, 11.5), g.pt(11.0, 12.0), g.pt(7.5, 12.0)],
                false,
                Color32::TRANSPARENT,
                Stroke::new(1.3 * g.s.clamp(1.0, 2.0), orange),
            ));
            g.poly(&[(12.0, 5.8), (13.8, 8.2), (10.2, 8.2)], orange);
            g.poly(&[(6.3, 12.0), (8.7, 10.2), (8.7, 13.8)], orange);
        }
        Icon::ResizeTable => {
            table(&g, 1.5, 1.5, 10.0, 9.0, c, light, paper, blue);
            dashed_rect(&g, 1.5, 1.5, 13.0, 13.0, light);
            arrow(&g, (9.5, 9.5), (14.5, 14.5), orange);
        }
        Icon::ConvertToRange => {
            table(&g, 1.0, 1.0, 8.0, 7.0, c, light, paper, blue);
            g.rect(7.0, 8.5, 8.0, 6.5, Some(paper), Some(c));
            g.stroke(&[(11.0, 8.5), (11.0, 15.0)], light);
            g.stroke(&[(7.0, 11.75), (15.0, 11.75)], light);
            g.line(&[(3.5, 8.5), (3.5, 12.0), (5.0, 12.0)], green, 1.3);
            g.poly(&[(6.8, 12.0), (4.8, 10.4), (4.8, 13.6)], green);
        }
        Icon::SelectData => {
            g.rect(1.5, 2.0, 13.0, 12.0, Some(paper), Some(light));
            for x in [5.8, 10.2] {
                g.stroke(&[(x, 2.0), (x, 14.0)], light);
            }
            for y in [6.0, 10.0] {
                g.stroke(&[(1.5, y), (14.5, y)], light);
            }
            g.rect(5.8, 6.0, 8.7, 8.0, Some(pal.wash(Cat::Green)), None);
            dashed_rect(&g, 5.8, 6.0, 8.7, 8.0, green);
        }
        Icon::CreateFromSelection => {
            g.rect(1.5, 5.0, 11.0, 9.5, Some(paper), Some(c));
            g.rect(1.5, 5.0, 11.0, 3.0, Some(pal.wash(Cat::Blue)), None);
            g.stroke(&[(1.5, 8.0), (12.5, 8.0)], c);
            g.stroke(&[(6.0, 5.0), (6.0, 14.5)], light);
            g.stroke(&[(1.5, 11.25), (12.5, 11.25)], light);
            tag(&g, 7.0, 0.8, orange, paper);
        }
        Icon::ArrangeAll => {
            for (x, y) in [(1.0, 1.5), (8.5, 1.5), (1.0, 8.5), (8.5, 8.5)] {
                window(&g, x, y, 6.5, 6.0, c, light, paper);
            }
        }
        Icon::Chart | Icon::ChartBar => columns(&g, 0.0, 0.0, 1.0, c, [blue, green, orange]),
        Icon::ChartLine => {
            g.stroke(&[(2.0, 14.0), (14.0, 14.0)], light);
            g.line(&[(2.0, 11.0), (5.5, 7.0), (8.5, 9.5), (13.5, 3.5)], blue, 1.5);
        }
        Icon::Sparkline => {
            // A small line chart inside a cell.
            g.rect(1.0, 4.0, 14.0, 8.0, Some(paper), Some(c));
            g.line(&[(2.5, 10.0), (5.0, 7.5), (7.0, 9.0), (9.5, 5.5), (11.5, 7.5), (13.5, 6.0)], blue, 1.3);
            g.circle(13.5, 6.0, 0.9, Some(red), None);
        }
        Icon::Forecast => {
            g.stroke(&[(1.5, 14.0), (14.5, 14.0)], light);
            g.poly(&[(9.0, 8.0), (14.5, 2.0), (14.5, 9.5)], pal.wash(Cat::Blue));
            g.line(&[(1.5, 11.5), (4.0, 9.0), (6.5, 10.5), (9.0, 8.0)], blue, 1.5);
            for (a, b) in [((9.0, 8.0), (10.6, 7.0)), ((11.6, 6.4), (13.2, 5.4))] {
                g.line(&[a, b], orange, 1.5);
            }
            g.circle(14.3, 4.8, 0.9, Some(orange), None);
        }
        Icon::ChartRecommended => {
            columns(&g, -1.5, 2.0, 0.85, c, [blue, green, orange]);
            star(&g, 12.0, 4.0, 3.4, Some(yellow), c);
        }
        Icon::ChartHistogram => {
            g.stroke(&[(1.5, 14.0), (14.5, 14.0)], c);
            for (i, h) in [3.0f32, 7.0, 10.0, 7.5, 4.0].iter().enumerate() {
                let x = 2.0 + i as f32 * 2.4;
                g.rect(x, 14.0 - h, 2.4, *h, Some(blue), Some(paper));
            }
        }
        Icon::AddChartElement => {
            columns(&g, -1.5, 2.0, 0.85, c, [blue, green, orange]);
            plus_badge(&g, 12.0, 4.0, green, paper);
        }
        Icon::ChangeChartType => {
            columns(&g, -1.5, 2.0, 0.85, c, [blue, green, orange]);
            g.circle(12.0, 4.0, 3.3, Some(blue), Some(paper));
            g.poly(&[(12.0, 4.0), (12.0, 0.7), (15.3, 4.0)], orange);
        }
        Icon::DataAnalysis => {
            columns(&g, -1.5, 0.0, 0.9, c, [blue, green, orange]);
            magnifier(&g, 10.5, 9.5, 3.0, c, Some(paper));
        }
        Icon::ChartTitle => {
            g.rect(2.5, 1.0, 11.0, 2.8, Some(c), None);
            columns(&g, 0.0, 3.0, 0.8, c, [blue, green, orange]);
        }
        Icon::ChartTreemap => {
            g.rect(1.5, 2.0, 7.0, 12.0, Some(blue), Some(paper));
            g.rect(8.5, 2.0, 6.0, 6.0, Some(orange), Some(paper));
            g.rect(8.5, 8.0, 3.0, 6.0, Some(green), Some(paper));
            g.rect(11.5, 8.0, 3.0, 6.0, Some(purple), Some(paper));
        }
        Icon::ChartPie => {
            g.circle(8.0, 8.0, 6.0, Some(blue), None);
            g.poly(&[(8.0, 8.0), (8.0, 2.0), (12.0, 3.4), (14.0, 8.0)], orange);
        }
        Icon::ChartScatter => {
            g.stroke(&[(2.0, 2.0), (2.0, 14.0), (14.0, 14.0)], c);
            for (x, y) in [(5.0, 11.0), (7.0, 8.0), (9.5, 9.5), (11.0, 5.0), (13.0, 4.0)] {
                g.circle(x, y, 1.1, Some(blue), None);
            }
        }
        Icon::Picture => {
            g.rect(2.0, 3.0, 12.0, 10.0, Some(pal.wash(Cat::Blue)), Some(c));
            g.poly(&[(3.0, 12.0), (7.0, 7.0), (10.0, 10.0), (11.5, 8.5), (13.0, 12.0)], green);
            g.circle(11.0, 5.5, 1.2, Some(yellow), None);
        }
        Icon::Shapes => {
            g.circle(5.5, 5.5, 3.5, Some(blue), None);
            g.rect(7.0, 7.0, 7.0, 7.0, Some(orange), None);
        }
        Icon::SelectObjects => {
            dashed_rect(&g, 1.5, 1.5, 11.0, 9.0, light);
            g.circle(4.5, 4.5, 1.6, Some(blue), None);
            g.rect(8.0, 6.0, 3.0, 3.0, Some(orange), None);
            pointer(&g, 7.0, 6.5, c, paper);
        }
        Icon::InkToShape => {
            g.line(&[(1.0, 6.0), (2.5, 3.0), (4.0, 7.0), (5.5, 2.5), (7.0, 6.5)], blue, 1.3);
            arrow(&g, (5.0, 9.0), (8.5, 11.5), c);
            g.poly(&[(12.0, 7.0), (15.0, 13.5), (9.0, 13.5)], orange);
        }
        Icon::Link => {
            g.line(&[(6.0, 10.0), (10.0, 6.0)], c, 1.2);
            g.rect(2.0, 8.0, 6.0, 4.0, None, Some(c));
            g.rect(8.0, 4.0, 6.0, 4.0, None, Some(c));
        }
        Icon::NewComment => {
            bubble(&g, 1.0, 2.0, 12.0, 10.0, paper, c);
            plus_badge(&g, 12.0, 11.5, green, paper);
        }
        Icon::DeleteComment => {
            bubble(&g, 1.0, 2.0, 12.0, 10.0, paper, c);
            red_x(&g, 12.0, 11.5, red);
        }
        Icon::ShowComments => {
            bubble(&g, 5.0, 1.0, 15.0, 7.5, pal.wash(Cat::Blue), c);
            bubble(&g, 1.0, 5.5, 11.0, 12.0, paper, c);
        }
        Icon::PrevComment | Icon::NextComment => {
            bubble(&g, 1.0, 1.5, 15.0, 10.0, paper, c);
            let (a, b) = if icon == Icon::NextComment { (4.5, 11.5) } else { (11.5, 4.5) };
            arrow(&g, (a, 5.75), (b, 5.75), blue);
        }
        Icon::Comment | Icon::Note => {
            g.poly(&[(2.0, 3.0), (14.0, 3.0), (14.0, 11.0), (2.0, 11.0)], if icon == Icon::Note { pal.wash(Cat::Amber) } else { paper });
            g.line(&[(2.0, 3.0), (14.0, 3.0), (14.0, 11.0), (7.0, 11.0), (4.0, 14.0), (4.0, 11.0), (2.0, 11.0), (2.0, 3.0)], c, 1.1);
        }
        Icon::TextBox => {
            g.rect(2.0, 3.0, 12.0, 10.0, None, Some(c));
            g.text(8.0, 8.0, "A", 8.0, c, (false, false));
        }
        Icon::Symbol => g.text(8.0, 8.0, "Ω", 12.0, c, (false, false)),
        Icon::Equation => {
            g.line(&[(1.5, 9.0), (3.5, 8.0), (5.5, 13.5), (8.5, 2.5), (14.5, 2.5)], c, 1.2);
            g.text(11.0, 8.5, "x", 8.0, blue, (false, true));
        }
        Icon::Function => g.text(8.0, 8.0, "fx", 11.0, c, (false, true)),
        Icon::UseInFormula => {
            g.text(5.5, 6.0, "fx", 9.0, c, (false, true));
            tag(&g, 6.5, 9.0, orange, paper);
        }
        Icon::EvaluateFormula => {
            g.text(5.5, 6.0, "fx", 9.0, c, (false, true));
            magnifier(&g, 10.5, 10.0, 3.0, c, Some(paper));
        }
        Icon::ShowFormulas => {
            g.rect(1.5, 2.5, 13.0, 11.0, Some(paper), Some(light));
            g.stroke(&[(1.5, 6.0), (14.5, 6.0)], light);
            g.stroke(&[(5.5, 2.5), (5.5, 13.5)], light);
            g.line(&[(8.0, 8.8), (12.5, 8.8)], blue, 1.4);
            g.line(&[(8.0, 11.2), (12.5, 11.2)], blue, 1.4);
        }
        Icon::Solver => {
            g.circle(7.0, 9.0, 5.5, Some(paper), Some(c));
            g.circle(7.0, 9.0, 3.4, None, Some(red));
            g.circle(7.0, 9.0, 1.3, Some(red), None);
            g.line(&[(7.0, 9.0), (14.0, 2.0)], c, 1.2);
            g.poly(&[(14.8, 1.2), (14.8, 4.2), (11.8, 1.2)], c);
        }
        Icon::Name => {
            g.rect(1.5, 4.0, 13.0, 8.0, None, Some(c));
            g.text(8.0, 8.0, "ab", 8.0, c, (false, false));
        }
        Icon::DefineName => {
            g.rect(1.0, 2.5, 11.0, 7.0, None, Some(c));
            g.text(6.5, 6.0, "ab", 7.0, c, (false, false));
            pencil(&g, 7.5, 15.0, orange, c);
        }
        Icon::Trace => {
            g.circle(4.0, 12.0, 1.6, Some(blue), None);
            g.circle(12.0, 4.0, 1.6, Some(blue), None);
            g.line(&[(5.0, 11.0), (11.0, 5.0)], blue, 1.2);
            g.poly(&[(12.5, 3.5), (9.0, 4.5), (11.5, 7.0)], blue);
        }
        Icon::TracePrecedents => {
            // Cells feeding the cell on the right: arrows run into it.
            g.rect(9.5, 5.5, 5.5, 5.0, Some(paper), Some(c));
            for y in [2.5, 13.5] {
                g.circle(2.0, y, 1.3, Some(blue), None);
                arrow(&g, (3.0, y + if y < 8.0 { 0.8 } else { -0.8 }), (9.0, if y < 8.0 { 6.8 } else { 9.2 }), blue);
            }
        }
        Icon::TraceDependents => {
            // The cell on the left feeding others: arrows run out of it.
            g.rect(1.0, 5.5, 5.5, 5.0, Some(paper), Some(c));
            for y in [2.5, 13.5] {
                arrow(&g, (7.0, if y < 8.0 { 6.8 } else { 9.2 }), (13.0, y + if y < 8.0 { 0.8 } else { -0.8 }), red);
                g.rect(13.2, y - 1.3, 2.4, 2.6, None, Some(red));
            }
        }
        Icon::RemoveArrows => {
            g.circle(2.5, 12.5, 1.4, Some(blue), None);
            arrow(&g, (3.5, 11.5), (10.0, 5.0), blue);
            red_x(&g, 12.0, 12.0, red);
        }
        Icon::Calc => calculator(&g, 3.0, c, light),
        Icon::CalcNow => {
            calculator(&g, 1.0, c, light);
            refresh(&g, 11.5, 11.5, 3.0, green, paper);
        }
        Icon::CalcSheet => {
            g.rect(1.0, 1.5, 12.0, 10.5, Some(paper), Some(c));
            for x in [5.0, 9.0] {
                g.stroke(&[(x, 1.5), (x, 12.0)], light);
            }
            for y in [5.0, 8.5] {
                g.stroke(&[(1.0, y), (13.0, y)], light);
            }
            refresh(&g, 11.5, 11.5, 3.0, green, paper);
        }
        Icon::Statistics => {
            g.rect(2.5, 1.5, 11.0, 13.0, Some(paper), Some(c));
            g.text(8.0, 5.5, "123", 5.5, blue, (true, false));
            for y in [9.5, 12.0] {
                g.stroke(&[(4.5, y), (11.5, y)], light);
            }
        }
        Icon::WhatIf => {
            g.rect(1.0, 3.0, 9.0, 10.0, Some(paper), Some(light));
            g.stroke(&[(1.0, 6.3), (10.0, 6.3)], light);
            g.stroke(&[(1.0, 9.6), (10.0, 9.6)], light);
            g.stroke(&[(5.5, 3.0), (5.5, 13.0)], light);
            g.text(11.5, 8.5, "?", 13.0, blue, (true, false));
        }
        Icon::Filter => funnel(&g, 0.0, 0.0, 1.0, c),
        Icon::FilterAdvanced => {
            funnel(&g, -1.0, -1.0, 0.75, c);
            g.rect(8.5, 8.5, 6.5, 6.0, Some(paper), Some(c));
            g.rect(8.5, 8.5, 6.5, 2.0, Some(blue), None);
            g.stroke(&[(8.5, 12.5), (15.0, 12.5)], light);
            g.stroke(&[(11.75, 10.5), (11.75, 14.5)], light);
        }
        Icon::ClearFilter => {
            funnel(&g, -1.0, -1.0, 0.85, c);
            red_x(&g, 12.0, 12.0, red);
        }
        Icon::Reapply => {
            funnel(&g, -1.0, -1.0, 0.85, c);
            refresh(&g, 11.5, 11.5, 3.0, green, paper);
        }
        Icon::Slicer => {
            g.rect(2.0, 1.0, 12.0, 14.0, Some(paper), Some(c));
            g.stroke(&[(3.5, 3.2), (9.0, 3.2)], c);
            g.rect(3.5, 5.0, 9.0, 2.6, Some(blue), None);
            g.rect(3.5, 8.4, 9.0, 2.6, Some(pal.wash(Cat::Blue)), Some(light));
            g.rect(3.5, 11.8, 9.0, 2.0, Some(pal.wash(Cat::Blue)), Some(light));
        }
        Icon::SortCustom => {
            g.rect(1.0, 1.5, 14.0, 13.0, Some(paper), Some(c));
            g.rect(2.5, 3.5, 7.0, 2.2, Some(blue), None);
            g.rect(2.5, 7.0, 5.5, 2.2, Some(green), None);
            g.rect(2.5, 10.5, 4.0, 2.2, Some(orange), None);
            g.stroke(&[(12.0, 3.5), (12.0, 12.0)], c);
            g.poly(&[(10.3, 10.5), (13.7, 10.5), (12.0, 13.3)], c);
        }
        Icon::SwitchRowCol => {
            g.rect(6.5, 6.5, 8.5, 8.5, Some(paper), Some(c));
            g.rect(6.5, 6.5, 8.5, 2.5, Some(pal.wash(Cat::Blue)), None);
            g.stroke(&[(10.75, 6.5), (10.75, 15.0)], light);
            g.line(&[(3.0, 12.0), (3.0, 3.0), (12.0, 3.0)], orange, 1.3);
            g.poly(&[(3.0, 14.5), (1.2, 11.5), (4.8, 11.5)], orange);
            g.poly(&[(14.5, 3.0), (11.5, 1.2), (11.5, 4.8)], orange);
        }
        Icon::SortAsc | Icon::SortDesc => {
            let (a, b) = if icon == Icon::SortAsc { ("A", "Z") } else { ("Z", "A") };
            g.text(5.0, 4.5, a, 6.5, c, (false, false));
            g.text(5.0, 11.5, b, 6.5, c, (false, false));
            g.stroke(&[(11.0, 2.5), (11.0, 13.0)], blue);
            g.poly(&[(9.0, 11.0), (13.0, 11.0), (11.0, 14.0)], blue);
        }
        Icon::Duplicates => {
            g.rect(2.0, 2.0, 9.0, 4.0, Some(paper), Some(c));
            g.rect(5.0, 7.0, 9.0, 4.0, Some(paper), Some(c));
            g.line(&[(9.0, 12.0), (14.0, 15.0)], red, 1.4);
            g.line(&[(14.0, 12.0), (9.0, 15.0)], red, 1.4);
        }
        Icon::TextColumns => {
            g.rect(1.5, 3.0, 5.5, 10.0, None, Some(c));
            g.rect(9.0, 3.0, 5.5, 10.0, None, Some(c));
            g.poly(&[(7.0, 6.0), (9.0, 8.0), (7.0, 10.0)], blue);
        }
        Icon::Accessibility => {
            g.circle(8.0, 8.0, 7.0, None, Some(c));
            g.circle(8.0, 4.3, 1.3, Some(blue), None);
            g.line(&[(3.8, 6.5), (12.2, 6.5)], blue, 1.3);
            g.line(&[(8.0, 6.5), (8.0, 9.5)], blue, 1.4);
            g.line(&[(5.8, 13.0), (8.0, 9.5), (10.2, 13.0)], blue, 1.3);
        }
        Icon::AllowEdit => {
            g.rect(1.0, 1.5, 11.0, 10.0, Some(paper), Some(light));
            g.stroke(&[(1.0, 5.0), (12.0, 5.0)], light);
            g.stroke(&[(1.0, 8.2), (12.0, 8.2)], light);
            g.stroke(&[(5.5, 1.5), (5.5, 11.5)], light);
            g.rect(5.5, 5.0, 6.5, 3.2, Some(pal.wash(Cat::Green)), Some(green));
            pencil(&g, 8.0, 15.0, orange, c);
        }
        Icon::ErrorCheck => {
            g.poly(&[(8.0, 1.5), (15.0, 14.0), (1.0, 14.0)], yellow);
            g.closed(&[(8.0, 1.5), (15.0, 14.0), (1.0, 14.0)], c, 1.0);
            g.line(&[(8.0, 5.8), (8.0, 10.0)], c, 1.5);
            g.circle(8.0, 12.0, 0.85, Some(c), None);
        }
        Icon::Validation => {
            g.rect(2.0, 3.0, 12.0, 10.0, None, Some(c));
            g.line(&[(4.0, 8.0), (6.5, 10.5), (11.5, 5.5)], green, 1.4);
        }
        Icon::Group | Icon::Ungroup | Icon::Subtotal => {
            g.stroke(&[(3.0, 3.0), (3.0, 13.0), (6.0, 13.0)], c);
            for y in [4.0, 7.0, 10.0] {
                g.stroke(&[(7.0, y), (14.0, y)], light);
            }
            match icon {
                Icon::Group => g.text(3.0, 2.5, "+", 6.0, blue, (true, false)),
                Icon::Ungroup => g.text(3.0, 2.5, "−", 6.0, blue, (true, false)),
                _ => g.text(11.0, 13.0, "Σ", 6.0, blue, (false, false)),
            }
        }
        Icon::Lock => lock(&g, 3.5, 2.5, 1.0, yellow, c),
        Icon::ProtectWorkbook => {
            g.rect(1.0, 1.0, 10.5, 11.0, Some(green), None);
            g.rect(2.8, 2.8, 6.9, 7.4, Some(paper), None);
            g.stroke(&[(6.25, 2.8), (6.25, 10.2)], green);
            g.stroke(&[(2.8, 6.5), (9.7, 6.5)], green);
            lock(&g, 8.5, 7.0, 0.7, yellow, c);
        }
        Icon::Spell => {
            g.text(7.0, 6.0, "abc", 7.0, c, (false, false));
            g.line(&[(4.0, 11.0), (6.5, 13.5), (12.0, 8.5)], green, 1.4);
        }
        Icon::Freeze => {
            g.rect(2.0, 2.5, 12.0, 11.0, None, Some(c));
            g.rect(2.0, 2.5, 12.0, 3.0, Some(pal.wash(Cat::Blue)), None);
            g.rect(2.0, 2.5, 3.5, 11.0, Some(pal.wash(Cat::Blue)), None);
            g.line(&[(2.0, 5.5), (14.0, 5.5)], blue, 1.2);
            g.line(&[(5.5, 2.5), (5.5, 13.5)], blue, 1.2);
        }
        Icon::IconsGallery => {
            star(&g, 4.5, 4.5, 3.5, Some(yellow), c);
            g.circle(11.5, 4.5, 3.0, Some(green), None);
            g.line(&[(10.0, 4.6), (11.2, 5.8), (13.2, 3.4)], paper, 1.2);
            g.circle(4.5, 9.8, 1.4, Some(c), None);
            g.poly(&[(1.8, 14.8), (2.6, 12.0), (6.4, 12.0), (7.2, 14.8)], c);
            g.poly(&[(11.5, 8.5), (15.0, 11.5), (8.0, 11.5)], orange);
            g.rect(9.0, 11.5, 5.0, 3.5, Some(orange), None);
        }
        Icon::Theme => {
            g.rect(2.0, 2.0, 5.5, 5.5, Some(blue), None);
            g.rect(8.5, 2.0, 5.5, 5.5, Some(orange), None);
            g.rect(2.0, 8.5, 5.5, 5.5, Some(green), None);
            g.rect(8.5, 8.5, 5.5, 5.5, Some(purple), None);
        }
        Icon::Margins => {
            g.rect(3.0, 1.5, 10.0, 13.0, None, Some(c));
            g.rect(5.0, 3.5, 6.0, 9.0, None, Some(blue));
        }
        Icon::Orient => {
            g.rect(2.0, 4.0, 8.0, 10.0, None, Some(c));
            g.rect(6.0, 2.0, 8.0, 6.0, Some(paper), Some(blue));
        }
        Icon::SheetRtl => {
            // A sheet with an arrow running right to left.
            g.rect(2.0, 2.5, 12.0, 8.0, None, Some(c));
            g.stroke(&[(6.0, 2.5), (6.0, 10.5)], light);
            g.stroke(&[(10.0, 2.5), (10.0, 10.5)], light);
            g.line(&[(13.5, 13.0), (4.5, 13.0)], blue, 1.4);
            g.poly(&[(2.0, 13.0), (5.0, 10.5), (5.0, 15.5)], blue);
        }
        Icon::PrintArea => {
            g.rect(2.5, 2.5, 11.0, 11.0, None, Some(light));
            g.line(&[(4.0, 4.0), (11.0, 4.0), (11.0, 10.0), (4.0, 10.0), (4.0, 4.0)], blue, 1.4);
        }
        Icon::Script => {
            g.rect(2.0, 2.0, 12.0, 12.0, None, Some(c));
            g.line(&[(4.5, 6.0), (7.0, 8.0), (4.5, 10.0)], green, 1.3);
            g.stroke(&[(8.0, 10.5), (11.5, 10.5)], c);
        }
        Icon::CommandPalette => {
            let frame = Rect::from_min_max(g.pt(1.0, 4.0), g.pt(15.0, 12.0));
            g.p.rect_filled(frame, 2.0 * g.s, paper);
            g.p.rect_stroke(frame, 2.0 * g.s, Stroke::new(1.0, c), egui::StrokeKind::Inside);
            g.line(&[(3.5, 6.2), (5.5, 8.0), (3.5, 9.8)], green, 1.3);
            g.stroke(&[(7.0, 8.0), (12.5, 8.0)], light);
            g.stroke(&[(7.0, 6.0), (7.0, 10.0)], c);
        }
        Icon::AgentControl => {
            g.stroke(&[(8.0, 1.8), (8.0, 4.5)], c);
            g.circle(8.0, 1.6, 1.0, Some(blue), None);
            let head = Rect::from_min_max(g.pt(2.5, 4.5), g.pt(13.5, 13.0));
            g.p.rect_filled(head, 2.5 * g.s, paper);
            g.p.rect_stroke(head, 2.5 * g.s, Stroke::new(1.0, c), egui::StrokeKind::Inside);
            g.circle(5.8, 8.0, 1.2, Some(blue), None);
            g.circle(10.2, 8.0, 1.2, Some(blue), None);
            g.stroke(&[(6.0, 11.0), (10.0, 11.0)], c);
            g.stroke(&[(1.0, 7.5), (1.0, 10.0)], c);
            g.stroke(&[(15.0, 7.5), (15.0, 10.0)], c);
        }
        Icon::ActionJournal => {
            g.rect(2.0, 1.0, 10.0, 14.0, Some(paper), Some(c));
            for y in [4.0, 6.5, 9.0] {
                g.stroke(&[(4.0, y), (10.0, y)], light);
            }
            g.circle(11.5, 11.5, 3.3, Some(paper), Some(c));
            g.stroke(&[(11.5, 9.6), (11.5, 11.5), (13.0, 12.3)], blue);
        }
        Icon::CheckUpdates => {
            refresh(&g, 8.0, 8.0, 6.0, green, paper);
            g.line(&[(8.0, 4.5), (8.0, 10.0)], c, 1.3);
            g.poly(&[(5.8, 9.0), (10.2, 9.0), (8.0, 11.8)], c);
        }
        Icon::Pen => pen(&g, blue, c),
        Icon::PenBlack => pen(&g, Color32::from_gray(40), c),
        Icon::PenRed => pen(&g, red, c),
        Icon::Highlighter => {
            g.line(&[(1.0, 14.5), (9.0, 14.5)], yellow.gamma_multiply(0.7), 2.6);
            g.poly(&[(4.0, 12.5), (4.5, 9.5), (6.5, 11.5)], c);
            g.poly(&[(4.5, 9.5), (11.0, 3.0), (13.0, 5.0), (6.5, 11.5)], yellow);
            g.poly(&[(11.0, 3.0), (12.5, 1.5), (14.5, 3.5), (13.0, 5.0)], c);
        }
        Icon::InkEraser => {
            g.line(&[(1.0, 13.0), (3.0, 11.0), (5.0, 13.5), (7.0, 11.5)], blue, 1.3);
            g.line(&[(9.0, 13.5), (11.0, 11.5), (13.0, 13.5), (15.0, 11.5)], blue.gamma_multiply(0.35), 1.3);
            g.poly(&[(5.0, 8.0), (10.0, 3.0), (13.0, 6.0), (8.0, 11.0)], red.gamma_multiply(0.55));
            g.poly(&[(10.0, 3.0), (11.5, 1.5), (14.5, 4.5), (13.0, 6.0)], red);
        }
        Icon::Share => {
            g.line(&[(4.0, 7.0), (4.0, 13.0), (12.0, 13.0), (12.0, 7.0)], c, 1.2);
            g.line(&[(8.0, 10.0), (8.0, 2.5)], c, 1.2);
            g.line(&[(5.5, 5.0), (8.0, 2.5), (10.5, 5.0)], c, 1.2);
        }
        Icon::Settings => {
            // A gear: eight teeth around a ring.
            for k in 0..8 {
                let a = std::f32::consts::PI * k as f32 / 4.0;
                let (sx, sy) = (a.cos(), a.sin());
                g.line(&[(8.0 + 4.0 * sx, 8.0 + 4.0 * sy), (8.0 + 6.6 * sx, 8.0 + 6.6 * sy)], c, 2.2);
            }
            g.circle(8.0, 8.0, 4.6, Some(c), None);
            g.circle(8.0, 8.0, 2.0, Some(paper), None);
        }
        Icon::FormatPane => {
            g.rect(1.0, 2.0, 14.0, 12.0, Some(paper), Some(c));
            g.rect(9.0, 2.0, 6.0, 12.0, Some(pal.wash(Cat::Blue)), None);
            g.stroke(&[(9.0, 2.0), (9.0, 14.0)], c);
            for y in [5.0, 7.5, 10.0] {
                g.stroke(&[(10.3, y), (13.7, y)], blue);
            }
            columns(&g, -1.0, 3.0, 0.5, c, [blue, green, orange]);
        }
        Icon::Appearance => {
            g.circle(8.0, 8.0, 5.5, None, Some(c));
            let half: Vec<(f32, f32)> = (0..=12)
                .map(|i| {
                    let a = std::f32::consts::PI * (i as f32 / 12.0 - 0.5);
                    (8.0 + 5.5 * a.cos(), 8.0 + 5.5 * a.sin())
                })
                .collect();
            g.poly(&half, c);
        }
        Icon::Book => {
            g.rect(2.0, 2.0, 12.0, 12.0, Some(green), None);
            g.rect(4.0, 4.0, 8.0, 8.0, Some(paper), None);
            g.stroke(&[(8.0, 4.0), (8.0, 12.0)], green);
            g.stroke(&[(4.0, 8.0), (12.0, 8.0)], green);
        }
        Icon::GetData => {
            g.poly(&[(1.0, 1.0), (6.0, 1.0), (8.0, 3.0), (8.0, 10.0), (1.0, 10.0)], paper);
            g.line(&[(1.0, 1.0), (6.0, 1.0), (8.0, 3.0), (8.0, 10.0), (1.0, 10.0), (1.0, 1.0)], c, 1.0);
            for y in [4.5, 6.5, 8.5] {
                g.stroke(&[(2.5, y), (6.5, y)], light);
            }
            g.rect(8.5, 9.0, 6.5, 6.0, Some(paper), Some(c));
            g.rect(8.5, 9.0, 6.5, 2.0, Some(green), None);
            g.stroke(&[(11.75, 11.0), (11.75, 15.0)], light);
            g.line(&[(4.5, 11.0), (4.5, 13.0), (6.0, 13.0)], green, 1.3);
            g.poly(&[(8.0, 13.0), (6.0, 11.5), (6.0, 14.5)], green);
        }
        Icon::FlashFill => {
            g.rect(1.0, 1.5, 7.0, 13.0, Some(paper), Some(light));
            for y in [4.75, 8.0, 11.25] {
                g.stroke(&[(1.0, y), (8.0, y)], light);
            }
            g.poly(&[(12.5, 1.0), (8.5, 8.5), (11.5, 8.5)], yellow);
            g.poly(&[(10.5, 7.0), (14.0, 7.0), (9.5, 15.0)], yellow);
            g.line(&[(12.5, 1.0), (8.5, 8.5), (11.0, 8.5), (9.5, 15.0), (14.0, 7.0), (11.5, 7.0), (12.5, 1.0)], orange, 0.8);
        }
        Icon::Person => {
            g.circle(8.0, 5.5, 3.0, None, Some(c));
            g.p.add(CubicBezierShape::from_points_stroke(
                [g.pt(2.5, 14.5), g.pt(2.5, 9.5), g.pt(13.5, 9.5), g.pt(13.5, 14.5)],
                false,
                Color32::TRANSPARENT,
                Stroke::new(1.15 * g.s.clamp(1.0, 2.0), c),
            ));
        }
        Icon::Folder => {
            g.poly(&[(1.5, 4.0), (6.0, 4.0), (7.5, 5.5), (14.5, 5.5), (14.5, 13.0), (1.5, 13.0)], yellow);
        }
        Icon::FnBook(glyph) => {
            let spine = pal.accent.unwrap_or(c);
            g.rect(3.0, 1.5, 10.5, 13.0, Some(paper), Some(c));
            g.rect(3.0, 1.5, 2.5, 13.0, Some(spine), None);
            let (x, y) = (9.5, 7.8);
            match glyph {
                FnGlyph::Star => {
                    let pts: Vec<(f32, f32)> = (0..10)
                        .map(|i| {
                            let a = std::f32::consts::PI * (i as f32 / 5.0 - 0.5);
                            let rad = if i % 2 == 0 { 3.4 } else { 1.4 };
                            (x + rad * a.cos(), y + rad * a.sin())
                        })
                        .collect();
                    g.closed(&pts, spine, 1.0);
                }
                FnGlyph::Coins => {
                    for k in 0..3 {
                        let top = y + 1.6 - k as f32 * 2.3;
                        g.p.rect_filled(Rect::from_min_max(g.pt(x - 2.8, top), g.pt(x + 2.8, top + 1.7)), 0.9 * g.s, spine);
                    }
                }
                FnGlyph::Question => g.text(x, y, "?", 8.5, spine, (true, false)),
                FnGlyph::Letter => g.text(x, y, "A", 8.0, spine, (true, false)),
                FnGlyph::Clock => {
                    g.circle(x, y, 3.2, None, Some(spine));
                    g.stroke(&[(x, y - 2.0), (x, y), (x + 1.6, y + 0.9)], spine);
                }
                FnGlyph::Magnifier => {
                    g.circle(x - 0.6, y - 0.6, 2.3, None, Some(spine));
                    g.line(&[(x + 1.0, y + 1.0), (x + 3.0, y + 3.0)], spine, 1.5);
                }
                FnGlyph::Theta => g.text(x, y, "θ", 8.5, spine, (false, false)),
                FnGlyph::Dots => {
                    for dx in [-2.4, 0.0, 2.4] {
                        g.circle(x + dx, y, 0.9, Some(spine), None);
                    }
                }
            }
        }
        Icon::Header | Icon::Footer => {
            g.rect(3.5, 1.5, 9.0, 13.0, None, Some(c));
            let y = if icon == Icon::Header { 2.5 } else { 11.0 };
            g.rect(4.5, y, 7.0, 2.5, Some(blue), None);
            for yy in if icon == Icon::Header { [7.0, 9.5] } else { [4.5, 7.0] } {
                g.stroke(&[(5.5, yy), (10.5, yy)], light);
            }
        }
        Icon::PageNumber | Icon::PageCount => {
            if icon == Icon::PageCount {
                g.rect(5.5, 1.0, 8.5, 11.5, None, Some(light));
            }
            g.rect(2.5, 3.0, 8.5, 12.0, Some(paper), Some(c));
            // A "#" mark.
            g.stroke(&[(5.3, 6.5), (4.8, 12.5)], blue);
            g.stroke(&[(8.3, 6.5), (7.8, 12.5)], blue);
            g.stroke(&[(4.0, 8.4), (9.6, 8.4)], blue);
            g.stroke(&[(3.8, 10.8), (9.4, 10.8)], blue);
        }
        Icon::Calendar => {
            g.rect(2.0, 3.0, 12.0, 11.0, Some(paper), Some(c));
            g.rect(2.0, 3.0, 12.0, 3.0, Some(blue), None);
            for (x, y) in [(4.5, 8.5), (7.5, 8.5), (10.5, 8.5), (4.5, 11.5), (7.5, 11.5)] {
                g.rect(x - 0.6, y - 0.6, 1.4, 1.4, Some(c), None);
            }
        }
        Icon::Clock => {
            let center = g.pt(8.0, 8.0);
            g.p.circle_stroke(center, 6.0 * g.s, Stroke::new(1.15 * g.s.clamp(1.0, 2.0), c));
            g.stroke(&[(8.0, 4.5), (8.0, 8.0), (10.8, 9.6)], blue);
        }
        Icon::FileName => {
            g.poly(&[(3.0, 1.5), (10.0, 1.5), (13.0, 4.5), (13.0, 14.5), (3.0, 14.5)], paper);
            g.line(&[(3.0, 1.5), (10.0, 1.5), (13.0, 4.5), (13.0, 14.5), (3.0, 14.5), (3.0, 1.5)], c, 1.15);
            g.stroke(&[(10.0, 1.5), (10.0, 4.5), (13.0, 4.5)], c);
            g.line(&[(5.0, 9.0), (11.0, 9.0)], blue, 1.4);
            g.stroke(&[(5.0, 11.5), (9.0, 11.5)], light);
        }
        Icon::Back => {
            g.circle(8.0, 8.0, 6.5, None, Some(c));
            g.line(&[(11.5, 8.0), (4.8, 8.0)], c, 1.2);
            g.line(&[(7.5, 5.3), (4.8, 8.0), (7.5, 10.7)], c, 1.2);
        }
        Icon::NewFile => {
            g.line(&[(9.5, 1.5), (3.5, 1.5), (3.5, 14.5), (12.5, 14.5), (12.5, 4.5), (9.5, 1.5), (9.5, 4.5), (12.5, 4.5)], c, 1.1);
        }
        Icon::NewWindow => {
            window(&g, 1.0, 2.0, 12.0, 10.5, c, light, paper);
            plus_badge(&g, 12.0, 12.0, green, paper);
        }
        Icon::SwitchWindows => {
            window(&g, 1.0, 1.0, 10.0, 8.5, c, light, paper);
            window(&g, 5.0, 6.0, 10.0, 8.5, c, blue, paper);
        }
        Icon::OpenFolder => {
            g.line(&[(13.0, 6.5), (13.0, 4.5), (7.5, 4.5), (6.0, 3.0), (1.5, 3.0), (1.5, 13.0)], c, 1.1);
            g.line(&[(1.5, 13.0), (4.0, 7.0), (15.0, 7.0), (12.5, 13.0), (1.5, 13.0)], c, 1.1);
        }
        Icon::Info => {
            g.circle(8.0, 8.0, 6.5, None, Some(c));
            g.line(&[(8.0, 7.0), (8.0, 11.8)], c, 1.3);
            g.circle(8.0, 4.8, 0.8, Some(c), None);
        }
        Icon::Print => {
            g.line(&[(4.5, 5.5), (4.5, 1.5), (11.5, 1.5), (11.5, 5.5)], c, 1.1);
            g.line(&[(4.0, 11.5), (1.5, 11.5), (1.5, 5.5), (14.5, 5.5), (14.5, 11.5), (12.0, 11.5)], c, 1.1);
            g.line(&[(4.5, 9.0), (4.5, 14.5), (11.5, 14.5), (11.5, 9.0), (4.5, 9.0)], c, 1.1);
            g.circle(12.3, 7.6, 0.6, Some(c), None);
        }
        Icon::Macro => {
            g.rect(2.5, 1.5, 9.0, 12.0, Some(paper), Some(c));
            for y in [4.5, 7.0, 9.5] {
                g.stroke(&[(4.5, y), (9.5, y)], light);
            }
            g.poly(&[(9.5, 8.5), (15.0, 11.75), (9.5, 15.0)], green);
        }
        Icon::MacroNew => {
            g.rect(2.5, 1.5, 9.0, 12.0, Some(paper), Some(c));
            for y in [4.5, 7.0, 9.5] {
                g.stroke(&[(4.5, y), (9.5, y)], light);
            }
            plus_badge(&g, 12.0, 12.0, green, paper);
        }
        Icon::FontGrow | Icon::FontShrink => {
            let grow = icon == Icon::FontGrow;
            g.text(6.0, 9.0, "A", if grow { 12.0 } else { 9.0 }, c, (false, false));
            if grow {
                g.poly(&[(13.0, 2.5), (15.5, 6.0), (10.5, 6.0)], blue);
            } else {
                g.poly(&[(10.5, 2.5), (15.5, 2.5), (13.0, 6.0)], blue);
            }
        }
        Icon::ShowDetail | Icon::HideDetail => {
            for y in [3.0, 6.5, 10.0, 13.5] {
                g.stroke(&[(8.5, y), (15.0, y)], light);
            }
            g.rect(1.0, 4.5, 6.0, 6.0, Some(paper), Some(c));
            g.line(&[(2.5, 7.5), (5.5, 7.5)], blue, 1.3);
            if icon == Icon::ShowDetail {
                g.line(&[(4.0, 6.0), (4.0, 9.0)], blue, 1.3);
            }
        }
        Icon::DeleteChart => {
            columns(&g, -1.5, 0.0, 0.9, c, [blue, green, orange]);
            red_x(&g, 12.0, 12.0, red);
        }
        Icon::PrintTitles => {
            for (x, y) in [(1.0, 1.0), (6.0, 5.0)] {
                g.rect(x, y, 9.0, 10.0, Some(paper), Some(c));
                g.rect(x, y, 9.0, 2.5, Some(blue), None);
                for k in [4.5, 7.0] {
                    g.stroke(&[(x + 1.5, y + k), (x + 7.5, y + k)], light);
                }
            }
        }
        Icon::Record => {
            g.circle(8.0, 8.0, 6.0, None, Some(c));
            g.circle(8.0, 8.0, 3.5, Some(pal.recording), None);
        }
        Icon::Relative => {
            g.rect(1.5, 1.5, 5.5, 5.0, Some(paper), Some(c));
            g.rect(9.0, 9.5, 5.5, 5.0, None, Some(c));
            g.line(&[(5.0, 7.5), (5.0, 12.0), (8.0, 12.0)], c, 1.1);
            g.poly(&[(7.5, 10.2), (9.6, 12.0), (7.5, 13.8)], c);
        }
        Icon::Collapse | Icon::Expand => {
            // A small sheet with a selected block and an arrow pointing up (collapse) or down.
            g.rect(2.0, 6.5, 12.0, 7.5, None, Some(c));
            g.rect(4.0, 9.0, 4.0, 3.0, Some(c), None);
            if icon == Icon::Collapse {
                g.poly(&[(8.0, 1.0), (11.5, 4.5), (4.5, 4.5)], c);
            } else {
                g.poly(&[(8.0, 5.5), (11.5, 2.0), (4.5, 2.0)], c);
            }
        }
        Icon::Play => g.poly(&[(4.0, 2.5), (13.0, 8.0), (4.0, 13.5)], green),
        Icon::Step => {
            g.line(&[(8.0, 1.5), (8.0, 9.0)], c, 1.3);
            g.poly(&[(4.8, 7.0), (11.2, 7.0), (8.0, 10.8)], c);
            g.line(&[(3.0, 13.5), (13.0, 13.5)], c, 1.3);
        }
        Icon::Continue => {
            g.rect(2.5, 2.5, 2.2, 11.0, Some(c), None);
            g.poly(&[(6.5, 2.5), (14.0, 8.0), (6.5, 13.5)], green);
        }
        Icon::RunTo => {
            g.line(&[(2.0, 5.0), (9.0, 5.0)], c, 1.3);
            g.poly(&[(8.0, 2.0), (12.0, 5.0), (8.0, 8.0)], c);
            g.line(&[(2.0, 11.5), (14.0, 11.5)], c, 1.6);
            g.line(&[(13.0, 3.0), (13.0, 9.0)], c, 1.1);
        }
        Icon::StopSquare => g.rect(3.0, 3.0, 10.0, 10.0, Some(pal.recording), None),
    }
}

/// A magnifier: a lens of radius `r` at (`x`, `y`) and its handle to the lower right.
fn magnifier(g: &G<'_>, x: f32, y: f32, r: f32, c: Color32, fill: Option<Color32>) {
    g.circle(x, y, r, fill, Some(c));
    let d = r * std::f32::consts::FRAC_1_SQRT_2;
    g.line(&[(x + d, y + d), (x + d + 4.0, y + d + 4.0)], c, 1.7);
}

/// A red cross centred on (`x`, `y`): delete, remove.
fn red_x(g: &G<'_>, x: f32, y: f32, red: Color32) {
    g.line(&[(x - 2.5, y - 2.5), (x + 2.5, y + 2.5)], red, 1.7);
    g.line(&[(x + 2.5, y - 2.5), (x - 2.5, y + 2.5)], red, 1.7);
}

/// A plus on a paper disc centred on (`x`, `y`): new, add.
fn plus_badge(g: &G<'_>, x: f32, y: f32, c: Color32, paper: Color32) {
    g.circle(x, y, 3.4, Some(paper), None);
    g.line(&[(x, y - 2.6), (x, y + 2.6)], c, 1.7);
    g.line(&[(x - 2.6, y), (x + 2.6, y)], c, 1.7);
}

/// An arrow from `a` to `b` with a filled head at `b`.
fn arrow(g: &G<'_>, a: (f32, f32), b: (f32, f32), c: Color32) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = (dx * dx + dy * dy).sqrt().max(0.01);
    let (ux, uy) = (dx / len, dy / len);
    let base = (b.0 - ux * 2.6, b.1 - uy * 2.6);
    g.line(&[a, base], c, 1.3);
    g.poly(&[b, (base.0 - uy * 1.6, base.1 + ux * 1.6), (base.0 + uy * 1.6, base.1 - ux * 1.6)], c);
}

/// A dashed rectangle outline (a selection).
fn dashed_rect(g: &G<'_>, x: f32, y: f32, w: f32, h: f32, c: Color32) {
    let corners = [(x, y), (x + w, y), (x + w, y + h), (x, y + h), (x, y)];
    for side in corners.windows(2) {
        let ((x0, y0), (x1, y1)) = (side[0], side[1]);
        let len = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        let n = (len / 2.0).floor().max(1.0) as usize;
        for k in (0..n).step_by(2) {
            let (t0, t1) = (k as f32 / n as f32, ((k + 1) as f32 / n as f32).min(1.0));
            g.line(&[(x0 + (x1 - x0) * t0, y0 + (y1 - y0) * t0), (x0 + (x1 - x0) * t1, y0 + (y1 - y0) * t1)], c, 1.1);
        }
    }
}

/// A circular arrow (recalculate, reapply, refresh) of radius `r` around (`x`, `y`) on a paper
/// disc.
fn refresh(g: &G<'_>, x: f32, y: f32, r: f32, c: Color32, paper: Color32) {
    g.circle(x, y, r + 0.8, Some(paper), None);
    let pts: Vec<(f32, f32)> = (0..=14)
        .map(|i| {
            let a = std::f32::consts::PI * (-0.35 + 1.55 * i as f32 / 14.0);
            (x + r * a.cos(), y + r * a.sin())
        })
        .collect();
    g.line(&pts, c, 1.3);
    let a = std::f32::consts::PI * -0.35;
    let (ex, ey) = (x + r * a.cos(), y + r * a.sin());
    let h = (r * 0.55).max(1.6);
    g.poly(&[(ex + h, ey - h * 0.2), (ex - h * 0.6, ey - h * 0.9), (ex - h * 0.2, ey + h)], c);
}

/// A speech bubble from (`x0`, `y0`) to (`x1`, `y1`) with its tail at the lower left.
fn bubble(g: &G<'_>, x0: f32, y0: f32, x1: f32, y1: f32, fill: Color32, c: Color32) {
    g.poly(&[(x0, y0), (x1, y0), (x1, y1), (x0, y1)], fill);
    let tx = x0 + 2.0;
    g.poly(&[(tx, y1), (tx + 3.0, y1), (tx, y1 + 3.0)], fill);
    g.line(&[(x0, y0), (x1, y0), (x1, y1), (tx + 3.0, y1), (tx, y1 + 3.0), (tx, y1), (x0, y1), (x0, y0)], c, 1.1);
}

/// A window with a title bar.
#[allow(clippy::too_many_arguments)]
fn window(g: &G<'_>, x: f32, y: f32, w: f32, h: f32, c: Color32, bar: Color32, paper: Color32) {
    g.rect(x, y, w, h, Some(paper), Some(c));
    g.rect(x, y, w, (h * 0.22).max(1.8), Some(bar), None);
}

/// A table: a grid with a coloured header row.
#[allow(clippy::too_many_arguments)]
fn table(g: &G<'_>, x: f32, y: f32, w: f32, h: f32, c: Color32, light: Color32, paper: Color32, header: Color32) {
    g.rect(x, y, w, h, Some(paper), Some(c));
    g.rect(x, y, w, h * 0.27, Some(header), None);
    g.stroke(&[(x + w / 3.0, y), (x + w / 3.0, y + h)], light);
    g.stroke(&[(x, y + h * 0.64), (x + w, y + h * 0.64)], light);
}

/// A three-column chart on a baseline, shifted by (`dx`, `dy`) and scaled by `k` around the
/// lower left.
fn columns(g: &G<'_>, dx: f32, dy: f32, k: f32, c: Color32, colors: [Color32; 3]) {
    let p = |x: f32, y: f32| (2.0 + dx + (x - 2.0) * k, 14.0 + (y - 14.0) * k + dy.min(0.0));
    let base = p(2.0, 14.0);
    let end = p(14.0, 14.0);
    g.stroke(&[base, end], c);
    for (i, (x, top)) in [(3.0, 8.0), (6.8, 4.0), (10.6, 6.0)].iter().enumerate() {
        let (x0, y0) = p(*x, *top);
        let (x1, _) = p(x + 2.5, 14.0);
        let y0 = y0.max(dy.max(0.0) + 0.5);
        g.rect(x0, y0, x1 - x0, base.1 - y0, Some(colors[i]), None);
    }
}

/// A funnel (filter), shifted by (`dx`, `dy`) and scaled by `k`.
fn funnel(g: &G<'_>, dx: f32, dy: f32, k: f32, c: Color32) {
    let f = |x: f32, y: f32| (x * k + dx + (1.0 - k) * 2.0, y * k + dy + (1.0 - k) * 2.0);
    g.poly(&[f(2.0, 3.0), f(14.0, 3.0), f(9.5, 8.5), f(6.5, 8.5)], c);
    g.poly(&[f(6.5, 8.5), f(9.5, 8.5), f(9.5, 13.5), f(6.5, 12.0)], c);
}

/// A five-pointed star outline (filled when `fill` is set) of radius `r` around (`x`, `y`).
fn star(g: &G<'_>, x: f32, y: f32, r: f32, fill: Option<Color32>, c: Color32) {
    let pts: Vec<(f32, f32)> = (0..10)
        .map(|i| {
            let a = std::f32::consts::PI * (i as f32 / 5.0 - 0.5);
            let rad = if i % 2 == 0 { r } else { r * 0.45 };
            (x + rad * a.cos(), y + rad * a.sin())
        })
        .collect();
    if let Some(f) = fill {
        // The star is concave: fill its centre pentagon and each point.
        let inner: Vec<(f32, f32)> = pts.iter().skip(1).step_by(2).copied().collect();
        g.poly(&inner, f);
        for i in (0..10).step_by(2) {
            g.poly(&[pts[(i + 9) % 10], pts[i], pts[i + 1]], f);
        }
    }
    g.closed(&pts, c, 0.9);
}

/// A name tag (label with a hole) with its left point at (`x`, `y`).
fn tag(g: &G<'_>, x: f32, y: f32, fill: Color32, paper: Color32) {
    g.poly(&[(x, y + 2.5), (x + 2.5, y), (x + 8.5, y), (x + 8.5, y + 5.0), (x + 2.5, y + 5.0)], fill);
    g.circle(x + 2.6, y + 2.5, 0.8, Some(paper), None);
}

/// A pencil whose point is at (`x`, `y`), slanting up to the right.
fn pencil(g: &G<'_>, x: f32, y: f32, body: Color32, c: Color32) {
    let p = |dx: f32, dy: f32| (x + dx, y + dy);
    g.poly(&[p(0.0, 0.0), p(0.6, -2.4), p(2.4, -0.6)], c);
    g.poly(&[p(0.6, -2.4), p(6.0, -7.8), p(7.8, -6.0), p(2.4, -0.6)], body);
}

/// A pen with its body in `body`: the ink colour it draws.
fn pen(g: &G<'_>, body: Color32, c: Color32) {
    g.poly(&[(3.0, 13.0), (4.0, 10.0), (6.0, 12.0)], c);
    g.poly(&[(4.0, 10.0), (11.0, 3.0), (13.0, 5.0), (6.0, 12.0)], body);
    g.closed(&[(4.0, 10.0), (11.0, 3.0), (13.0, 5.0), (6.0, 12.0)], c, 0.6);
    g.line(&[(1.5, 15.0), (5.0, 15.0)], body, 1.6);
}

/// A padlock whose shackle starts at (`x`, `y`), scaled by `k`.
fn lock(g: &G<'_>, x: f32, y: f32, k: f32, body: Color32, c: Color32) {
    let p = |dx: f32, dy: f32| (x + dx * k, y + dy * k);
    let (bx, by) = p(0.0, 4.5);
    g.rect(bx, by, 9.0 * k, 7.0 * k, Some(body), Some(c));
    g.line(&[p(2.0, 4.5), p(2.0, 2.0), p(4.5, 0.0), p(7.0, 2.0), p(7.0, 4.5)], c, 1.2);
}

/// A calculator whose left edge is at `x`.
fn calculator(g: &G<'_>, x: f32, c: Color32, light: Color32) {
    g.rect(x, 1.5, 10.0, 13.0, None, Some(c));
    g.rect(x + 1.5, 3.0, 7.0, 3.0, Some(light), None);
    for (dx, y) in [(2.5, 8.5), (5.0, 8.5), (7.5, 8.5), (2.5, 11.5), (5.0, 11.5), (7.5, 11.5)] {
        g.circle(x + dx, y, 0.8, Some(c), None);
    }
}

/// An arrow pointer with its tip at (`x`, `y`).
fn pointer(g: &G<'_>, x: f32, y: f32, c: Color32, paper: Color32) {
    let pts = [(x, y), (x, y + 8.0), (x + 2.2, y + 6.0), (x + 3.8, y + 9.0), (x + 5.0, y + 8.4), (x + 3.5, y + 5.5), (x + 6.0, y + 5.3)];
    g.poly(&[pts[0], pts[1], pts[2], pts[6]], paper);
    g.poly(&[pts[2], pts[3], pts[4], pts[5]], paper);
    g.closed(&pts, c, 0.9);
}

/// Icons offered by Insert › Icons, by name.
pub const LIBRARY: &[(&str, Icon)] = &[
    ("check", Icon::Check),
    ("close", Icon::Close),
    ("plus", Icon::Plus),
    ("minus", Icon::Minus),
    ("chart", Icon::Chart),
    ("line chart", Icon::ChartLine),
    ("pie chart", Icon::ChartPie),
    ("scatter", Icon::ChartScatter),
    ("table", Icon::Table),
    ("sum", Icon::Sum),
    ("filter", Icon::Filter),
    ("lock", Icon::Lock),
    ("comment", Icon::Comment),
    ("note", Icon::Note),
    ("picture", Icon::Picture),
    ("book", Icon::Book),
    ("folder", Icon::Folder),
    ("search", Icon::Search),
    ("calculator", Icon::Calc),
    ("link", Icon::Link),
    ("pen", Icon::Pen),
    ("share", Icon::Share),
    ("settings", Icon::Settings),
    ("home", Icon::Home),
    ("save", Icon::Save),
    ("theme", Icon::Theme),
    ("shapes", Icon::Shapes),
    ("text", Icon::TextBox),
    ("trace", Icon::Trace),
    ("validation", Icon::Validation),
];

pub fn from_name(name: &str) -> Option<Icon> {
    LIBRARY.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, i)| *i)
}

fn paint_alignment(g: &G<'_>, icon: Icon, c: Color32, light: Color32, blue: Color32) {
    if matches!(icon, Icon::AlignTop | Icon::AlignMiddle | Icon::AlignBottom) {
        let ys: [f32; 3] = match icon {
            Icon::AlignTop => [3.0, 5.5, 8.0],
            Icon::AlignMiddle => [5.5, 8.0, 10.5],
            _ => [8.0, 10.5, 13.0],
        };
        let edge = match icon {
            Icon::AlignTop => 1.5,
            Icon::AlignMiddle => -1.0,
            _ => 14.5,
        };
        if edge > 0.0 {
            g.stroke(&[(2.0, edge), (14.0, edge)], light);
        }
        for (i, y) in ys.iter().enumerate() {
            let w = if i == 1 { 8.0 } else { 10.0 };
            g.stroke(&[(8.0 - w / 2.0, *y), (8.0 + w / 2.0, *y)], c);
        }
    } else {
        for (i, y) in [3.5f32, 6.5, 9.5, 12.5].iter().enumerate() {
            let w = if i % 2 == 0 { 12.0 } else { 8.0 };
            let (x0, x1) = match icon {
                Icon::AlignLeft => (2.0, 2.0 + w),
                Icon::AlignRight => (14.0 - w, 14.0),
                Icon::AlignCenter => (8.0 - w / 2.0, 8.0 + w / 2.0),
                _ => (if i == 0 || i == 3 { 2.0 } else { 7.0 }, 14.0),
            };
            g.stroke(&[(x0, *y), (x1, *y)], c);
        }
        if icon == Icon::IndentInc {
            g.poly(&[(2.0, 6.0), (5.0, 8.0), (2.0, 10.0)], blue);
        }
        if icon == Icon::IndentDec {
            g.poly(&[(5.0, 6.0), (2.0, 8.0), (5.0, 10.0)], blue);
        }
    }
}
