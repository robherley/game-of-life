use crate::game::{Board, BoardError, Game};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use thiserror::Error;

const TABLE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS games (
    name TEXT PRIMARY KEY,
    board BLOB NOT NULL,
    generation INTEGER NOT NULL,
    delta INTEGER NOT NULL
)
"#;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("pool: {0}")]
    PoolError(r2d2::Error),
    #[error("sql: {0}")]
    SQLError(rusqlite::Error),
    #[error("zstd: {0}")]
    ZSTDError(String),
    #[error("board: {0}")]
    BoardError(BoardError),
}

impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> StoreError {
        StoreError::SQLError(error)
    }
}

impl From<r2d2::Error> for StoreError {
    fn from(error: r2d2::Error) -> StoreError {
        StoreError::PoolError(error)
    }
}

#[derive(Clone)]
pub struct Store {
    pub pool: Pool<SqliteConnectionManager>,
}

impl Store {
    pub fn new(db_path: String) -> Result<Self, StoreError> {
        let manager = SqliteConnectionManager::file(db_path);
        let pool = r2d2::Pool::new(manager)?;
        Ok(Self { pool })
    }

    pub fn migrate(&self) -> Result<(), StoreError> {
        let conn = self.conn()?;
        conn.execute(TABLE_SCHEMA, [])?;
        Ok(())
    }

    pub fn conn(&self) -> Result<PooledConnection<SqliteConnectionManager>, StoreError> {
        Ok(self.pool.get()?)
    }

    pub fn create(&self, name: &str, game: &Game) -> Result<(), StoreError> {
        let conn = self.conn()?;
        let mut stmt =
            conn.prepare("INSERT INTO games (name, board, generation, delta) VALUES (?, ?, ?, ?)")?;
        let compressed = Self::compress(game.board.to_string())?;
        stmt.execute(params![name, compressed, game.generation, game.delta])?;
        Ok(())
    }

    pub fn update(&self, name: &str, game: &Game) -> Result<(), StoreError> {
        let conn = self.conn()?;
        let mut stmt =
            conn.prepare("UPDATE games SET board = ?, generation = ?, delta = ? WHERE name = ?")?;
        let compressed = Self::compress(game.board.to_string())?;
        stmt.execute(params![compressed, game.generation, game.delta, name])?;
        Ok(())
    }

    pub fn find(&self, name: &str) -> Result<Option<Game>, StoreError> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT board, generation, delta FROM games WHERE name = ?")?;
        let mut rows = stmt.query([name])?;
        let row = match rows.next()? {
            Some(row) => row,
            None => return Ok(None),
        };
        let grid: Vec<u8> = row.get(0)?;
        let seed = Self::decompress(&grid)?;
        let board = Board::try_from(seed).map_err(|e| StoreError::BoardError(e))?;

        Ok(Some(Game {
            board,
            generation: row.get(1)?,
            delta: row.get(2)?,
        }))
    }

    fn compress(input: String) -> Result<Vec<u8>, StoreError> {
        zstd::encode_all(input.as_bytes(), 3)
            .map_err(|e| StoreError::ZSTDError(format!("unable to compress: {}", e)))
    }

    fn decompress(data: &[u8]) -> Result<String, StoreError> {
        let raw = zstd::decode_all(data)
            .map_err(|e| StoreError::ZSTDError(format!("unable to decompress: {}", e)))?;
        Ok(String::from_utf8(raw)
            .map_err(|e| StoreError::ZSTDError(format!("invalid utf8: {}", e)))?)
    }
}
