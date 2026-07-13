use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

pub struct DbSessao {
    conn: Arc<Mutex<Connection>>,
}

impl DbSessao {
    pub fn new(caminho_db: &str) -> Result<Self, String> {
        let conn = Connection::open(caminho_db)
            .map_err(|e| format!("Erro ao abrir DB: {}", e))?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS historico_erros (
                id INTEGER PRIMARY KEY,
                id_crianca TEXT NOT NULL,
                palavra TEXT NOT NULL,
                tentativas INTEGER NOT NULL
            )",
            [],
        ).map_err(|e| format!("Erro ao criar tabela: {}", e))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn salvar_erro(&self, id_crianca: &str, palavra: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap(); // Fallback seguro para Mutex local
        
        let mut stmt = conn.prepare("SELECT tentativas FROM historico_erros WHERE id_crianca = ?1 AND palavra = ?2")
            .map_err(|e| e.to_string())?;
        
        let mut rows = stmt.query([id_crianca, palavra]).map_err(|e| e.to_string())?;
        
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let tentativas: i32 = row.get(0).unwrap_or(0);
            conn.execute(
                "UPDATE historico_erros SET tentativas = ?1 WHERE id_crianca = ?2 AND palavra = ?3",
                rusqlite::params![tentativas + 1, id_crianca, palavra],
            ).map_err(|e| e.to_string())?;
        } else {
            conn.execute(
                "INSERT INTO historico_erros (id_crianca, palavra, tentativas) VALUES (?1, ?2, 1)",
                rusqlite::params![id_crianca, palavra],
            ).map_err(|e| e.to_string())?;
        }
        
        Ok(())
    }

    pub fn buscar_historico(&self, id_crianca: &str) -> Result<Vec<(String, i32)>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT palavra, tentativas FROM historico_erros WHERE id_crianca = ?1")
            .map_err(|e| e.to_string())?;
            
        let historico_iter = stmt.query_map([id_crianca], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).map_err(|e| e.to_string())?;
        
        let mut resultados = Vec::new();
        for item in historico_iter {
            if let Ok(tupla) = item {
                resultados.push(tupla);
            }
        }
        Ok(resultados)
    }
}
