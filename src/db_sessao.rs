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
        ).map_err(|e| format!("Erro ao criar tabela de erros: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS historico_chat (
                id INTEGER PRIMARY KEY,
                id_crianca TEXT NOT NULL,
                remetente TEXT NOT NULL,
                mensagem TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).map_err(|e| format!("Erro ao criar tabela de chat: {}", e))?;

        // Forçamos a reestruturação para suportar Missões (Game Loop) descartando a tabela antiga (já que é só estado)
        conn.execute("DROP TABLE IF EXISTS estado_jogo", []).ok();
        conn.execute(
            "CREATE TABLE estado_jogo (
                id_crianca TEXT PRIMARY KEY,
                palavra_desafio TEXT NOT NULL,
                acertos INTEGER NOT NULL DEFAULT 0,
                total INTEGER NOT NULL DEFAULT 3
            )",
            [],
        ).map_err(|e| format!("Erro ao criar tabela de estado: {}", e))?;

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

    pub fn salvar_mensagem(&self, id_crianca: &str, remetente: &str, mensagem: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO historico_chat (id_crianca, remetente, mensagem) VALUES (?1, ?2, ?3)",
            rusqlite::params![id_crianca, remetente, mensagem],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn obter_contexto(&self, id_crianca: &str, limite: i32) -> Result<String, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT remetente, mensagem FROM historico_chat 
             WHERE id_crianca = ?1 
             ORDER BY timestamp DESC LIMIT ?2"
        ).map_err(|e| e.to_string())?;
        
        let iter = stmt.query_map(rusqlite::params![id_crianca, limite], |row| {
            let remetente: String = row.get(0)?;
            let mensagem: String = row.get(1)?;
            Ok(format!("{}: {}", remetente, mensagem))
        }).map_err(|e| e.to_string())?;
        
        let mut mensagens = Vec::new();
        for item in iter {
            if let Ok(msg) = item {
                mensagens.push(msg);
            }
        }
        
        // Inverter para ficar na ordem cronológica (mais antiga primeiro)
        mensagens.reverse();
        Ok(mensagens.join("\n"))
    }

    pub fn iniciar_missao(&self, id_crianca: &str, palavra: &str, total: i32) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO estado_jogo (id_crianca, palavra_desafio, acertos, total) VALUES (?1, ?2, 0, ?3)
             ON CONFLICT(id_crianca) DO UPDATE SET palavra_desafio=excluded.palavra_desafio, acertos=0, total=excluded.total",
            rusqlite::params![id_crianca, palavra, total],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn avancar_missao(&self, id_crianca: &str, nova_palavra: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE estado_jogo SET palavra_desafio = ?1, acertos = acertos + 1 WHERE id_crianca = ?2",
            rusqlite::params![nova_palavra, id_crianca],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn obter_estado_missao(&self, id_crianca: &str) -> Option<(String, i32, i32)> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT palavra_desafio, acertos, total FROM estado_jogo WHERE id_crianca = ?1").ok()?;
        let mut rows = stmt.query([id_crianca]).ok()?;
        
        if let Some(row) = rows.next().ok()? {
            Some((
                row.get(0).unwrap_or_default(),
                row.get(1).unwrap_or(0),
                row.get(2).unwrap_or(3),
            ))
        } else {
            None
        }
    }

    pub fn limpar_desafio(&self, id_crianca: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM estado_jogo WHERE id_crianca = ?1",
            rusqlite::params![id_crianca],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}
