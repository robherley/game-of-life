use crate::game::Game;
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    writer::Writer,
};

pub struct TextOptions {
    pub alive: char,
    pub dead: char,
    pub separator: char,
}

impl TextOptions {
    pub fn new(alive: Option<char>, dead: Option<char>, separator: Option<char>) -> Self {
        Self {
            alive: alive.unwrap_or('#'),
            dead: dead.unwrap_or('.'),
            separator: separator.unwrap_or('\n'),
        }
    }
}

impl Default for TextOptions {
    fn default() -> Self {
        Self::new(None, None, None)
    }
}

pub fn text(game: &Game, opts: TextOptions) -> String {
    let board = &game.board;
    let mut result = String::with_capacity(board.rows() * board.cols() + board.rows());

    for (i, row) in board.grid.iter().enumerate() {
        for cell in row {
            result.push(if *cell { opts.alive } else { opts.dead });
        }
        if i < board.rows() - 1 {
            result.push(opts.separator);
        }
    }

    result
}

pub struct SVGOptions {
    pub cell_size: usize,
    pub stroke_width: usize,
    pub stroke_color: String,
    pub fill_color: String,
}

impl SVGOptions {
    pub fn new(
        cell_size: Option<usize>,
        stroke_width: Option<usize>,
        stroke_color: Option<String>,
        fill_color: Option<String>,
    ) -> Self {
        Self {
            cell_size: cell_size.unwrap_or(20),
            stroke_width: stroke_width.unwrap_or(2),
            stroke_color: stroke_color.unwrap_or("white".to_string()),
            fill_color: fill_color.unwrap_or("black".to_string()),
        }
    }
}

impl Default for SVGOptions {
    fn default() -> Self {
        Self::new(None, None, None, None)
    }
}

pub fn svg(game: &Game, opts: SVGOptions) -> Result<String, quick_xml::Error> {
    let board = &game.board;
    let width = board.cols() * opts.cell_size;
    let height = board.rows() * opts.cell_size + 20;

    let mut w = Writer::new(std::io::Cursor::new(Vec::<u8>::new()));

    w.write_event(Event::Start(BytesStart::new("svg").with_attributes(vec![
        ("xmlns", "http://www.w3.org/2000/svg"),
        ("width", &*format!("{}", width)),
        ("height", &*format!("{}", height)),
    ])))?;

    for (row, cells) in board.grid.iter().enumerate() {
        for (col, cell) in cells.iter().enumerate() {
            if *cell {
                w.write_event(Event::Empty(BytesStart::new("rect").with_attributes(vec![
                    ("x", &*format!("{}", col * opts.cell_size)),
                    ("y", &*format!("{}", row * opts.cell_size)),
                    ("width", &*format!("{}", opts.cell_size)),
                    ("height", &*format!("{}", opts.cell_size)),
                    ("fill", &opts.fill_color),
                    ("stroke", &opts.stroke_color),
                    ("stroke-width", &*format!("{}", opts.stroke_width)),
                ])))?;
            }
        }
    }

    w.write_event(Event::Start(BytesStart::new("text").with_attributes(vec![
        ("x", "50%"),
        ("y", &*format!("{}", height - 5)),
        ("font-family", "monospace"),
        ("font-size", "12"),
        ("fill", &opts.fill_color),
        ("dominant-baseline", "center"),
        ("text-anchor", "middle"),
    ])))?;
    w.write_event(Event::Text(BytesText::new(&*format!(
        "t = {}, Δ = {}",
        game.generation, game.delta
    ))))?;
    w.write_event(Event::End(BytesEnd::new("text")))?;

    w.write_event(Event::End(BytesEnd::new("svg")))?;
    Ok(std::str::from_utf8(&w.into_inner().into_inner())?.to_string())
}
