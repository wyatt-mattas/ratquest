use rusqlite::{params, Connection, Result};
use std::path::Path;
use thiserror::Error;

use crate::app::models::{ApiRequest, AuthDetails, AuthType, BasicAuth, RequestType};

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Database initialization error: {0}")]
    Init(String),
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, DatabaseError> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<(), DatabaseError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS groups (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS requests (
                id INTEGER PRIMARY KEY,
                group_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                request_type TEXT NOT NULL,
                url TEXT NOT NULL DEFAULT '',
                body TEXT NOT NULL DEFAULT '',
                auth_type TEXT NOT NULL DEFAULT 'None',
                auth_username TEXT,
                auth_password TEXT,
                FOREIGN KEY (group_id) REFERENCES groups(id),
                UNIQUE(group_id, name)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS headers (
                id INTEGER PRIMARY KEY,
                request_id INTEGER NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                FOREIGN KEY (request_id) REFERENCES requests(id)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS params (
                id INTEGER PRIMARY KEY,
                request_id INTEGER NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                FOREIGN KEY (request_id) REFERENCES requests(id)
            )",
            [],
        )?;

        Ok(())
    }

    pub fn create_group(&self, name: &str) -> Result<i64, DatabaseError> {
        self.conn
            .execute("INSERT INTO groups (name) VALUES (?1)", params![name])?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn create_request(
        &mut self,
        group_id: i64,
        request: &ApiRequest,
    ) -> Result<i64, DatabaseError> {
        let tx = self.conn.transaction()?;

        // Insert the main request
        tx.execute(
            "INSERT INTO requests (
                group_id, name, request_type, url, body,
                auth_type, auth_username, auth_password
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                group_id,
                request.name,
                request.request_type.as_str(),
                request.details.url,
                request.details.body,
                request.details.auth_type.as_str(),
                match &request.details.auth_details {
                    AuthDetails::Basic(auth) => Some(&auth.username),
                    AuthDetails::None => None,
                },
                match &request.details.auth_details {
                    AuthDetails::Basic(auth) => Some(&auth.password),
                    AuthDetails::None => None,
                },
            ],
        )?;

        let request_id = tx.last_insert_rowid();

        // Insert headers
        for (key, value) in &request.details.headers {
            tx.execute(
                "INSERT INTO headers (request_id, key, value) VALUES (?1, ?2, ?3)",
                params![request_id, key, value],
            )?;
        }

        // Insert parameters
        for (key, value) in &request.details.params {
            tx.execute(
                "INSERT INTO params (request_id, key, value) VALUES (?1, ?2, ?3)",
                params![request_id, key, value],
            )?;
        }

        tx.commit()?;
        Ok(request_id)
    }

    pub fn get_request(&self, id: i64) -> Result<ApiRequest, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT name, request_type, url, body, auth_type,
                    auth_username, auth_password
             FROM requests WHERE id = ?1",
        )?;

        let mut request = stmt.query_row(params![id], |row| {
            let name: String = row.get(0)?;
            let request_type: String = row.get(1)?;
            let url: String = row.get(2)?;
            let body: String = row.get(3)?;
            let auth_type: String = row.get(4)?;
            let auth_username: Option<String> = row.get(5)?;
            let auth_password: Option<String> = row.get(6)?;

            let mut request = ApiRequest::new(
                name,
                match request_type.as_str() {
                    "GET" => RequestType::GET,
                    "POST" => RequestType::POST,
                    "PUT" => RequestType::PUT,
                    "DELETE" => RequestType::DELETE,
                    "PATCH" => RequestType::PATCH,
                    _ => RequestType::GET,
                },
            );

            request.details.url = url;
            request.details.body = body;
            request.details.auth_type = match auth_type.as_str() {
                "Basic" => AuthType::Basic,
                _ => AuthType::None,
            };

            if let (Some(username), Some(password)) = (auth_username, auth_password) {
                request.details.auth_details = AuthDetails::Basic(BasicAuth { username, password });
            }

            Ok(request)
        })?;

        // Load headers
        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM headers WHERE request_id = ?1")?;
        let headers = stmt.query_map(params![id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for header in headers {
            let (key, value) = header?;
            request.details.headers.insert(key, value);
        }

        // Load parameters
        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM params WHERE request_id = ?1")?;
        let params = stmt.query_map(params![id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for param in params {
            let (key, value) = param?;
            request.details.params.insert(key, value);
        }

        Ok(request)
    }

    pub fn update_request(&mut self, id: i64, request: &ApiRequest) -> Result<(), DatabaseError> {
        let tx = self.conn.transaction()?;

        tx.execute(
            "UPDATE requests SET
                name = ?2,
                request_type = ?3,
                url = ?4,
                body = ?5,
                auth_type = ?6,
                auth_username = ?7,
                auth_password = ?8
             WHERE id = ?1",
            params![
                id,
                request.name,
                request.request_type.as_str(),
                request.details.url,
                request.details.body,
                request.details.auth_type.as_str(),
                match &request.details.auth_details {
                    AuthDetails::Basic(auth) => Some(&auth.username),
                    AuthDetails::None => None,
                },
                match &request.details.auth_details {
                    AuthDetails::Basic(auth) => Some(&auth.password),
                    AuthDetails::None => None,
                },
            ],
        )?;

        // Update headers
        tx.execute("DELETE FROM headers WHERE request_id = ?1", params![id])?;
        for (key, value) in &request.details.headers {
            tx.execute(
                "INSERT INTO headers (request_id, key, value) VALUES (?1, ?2, ?3)",
                params![id, key, value],
            )?;
        }

        // Update parameters
        tx.execute("DELETE FROM params WHERE request_id = ?1", params![id])?;
        for (key, value) in &request.details.params {
            tx.execute(
                "INSERT INTO params (request_id, key, value) VALUES (?1, ?2, ?3)",
                params![id, key, value],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn delete_request(&mut self, id: i64) -> Result<(), DatabaseError> {
        let tx = self.conn.transaction()?;

        tx.execute("DELETE FROM headers WHERE request_id = ?1", params![id])?;
        tx.execute("DELETE FROM params WHERE request_id = ?1", params![id])?;
        tx.execute("DELETE FROM requests WHERE id = ?1", params![id])?;

        tx.commit()?;
        Ok(())
    }

    pub fn get_all_groups(&self) -> Result<Vec<(i64, String)>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM groups ORDER BY name")?;
        let groups = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(groups)
    }

    pub fn get_requests_for_group(&self, group_id: i64) -> Result<Vec<ApiRequest>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM requests WHERE group_id = ?1 ORDER BY name")?;

        let request_ids = stmt.query_map(params![group_id], |row| row.get(0))?;
        let mut requests = Vec::new();

        for id_result in request_ids {
            let id = id_result?;
            let request = self.get_request(id)?;
            requests.push(request);
        }

        Ok(requests)
    }
}
