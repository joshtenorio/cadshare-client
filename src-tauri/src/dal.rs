use sqlx::{Pool, Row, Sqlite};
use std::path::Path;
use std::result::Result::Ok;
use crate::types::{ChangeType, UpdatedFile};

pub struct DataAccessLayer<'a> {
    pub pool: &'a Pool<Sqlite>
}

impl<'a> DataAccessLayer<'a> {
    pub fn new(user_pool: &'a Pool<Sqlite>) -> Self {
        DataAccessLayer { pool: &user_pool }
    }

    /// gets the current server URL to use for network calls
    pub async fn get_current_server(&self) -> Result<String, ()> {
        let output = sqlx::query("SELECT CASE WHEN debug_active = 1 THEN debug_url ELSE url END as url FROM server WHERE active = 1")
            .fetch_one(self.pool).await;
    
        match output {
            Ok(row) => Ok(row.get::<String, &str>("url")),
            Err(err) => {
                log::error!("couldn't get the current server url: {}", err);
                Ok("".to_string())
            }
        }
    }

    /// gets the current server URL for db foreign key purposes
    pub async fn get_active_server(&self) -> Result<String, ()> {
        let output = sqlx::query("SELECT url FROM server WHERE active = 1")
            .fetch_one(self.pool)
            .await;
    
        match output {
            Ok(row) => Ok(row.get::<String, &str>("url")),
            Err(err) => {
                log::error!("couldn't get the active server url: {}", err);
                Ok("".to_string())
            }
        }
    }

    /// deletes an entry from the file table
    pub async fn delete_file_entry(&self, pid: i32, path: String) -> Result<bool, ()> {
        let _ = sqlx::query(
            "DELETE FROM file
            WHERE pid = $1 AND filepath = $2",
        )
        .bind(pid)
        .bind(path)
        .execute(self.pool)
        .await;
        Ok(true)
    }

    /// get the project directory for the specified project
    pub async fn get_project_dir(&self, pid: i32) -> Result<String, ()> {
        //println!("current allocating {}B", get_allocated());
        let server = self.get_active_server().await.unwrap();
        let db_call = sqlx::query("SELECT server.local_dir, project.title, project.team_name FROM server, project WHERE server.active = 1 AND project.url = ? AND project.pid = ?")
            .bind(server)
            .bind(pid)
            .fetch_one(self.pool)
            .await;
        match db_call {
            Ok(row) => {
                let output = Path::new(&row.get::<String, &str>("local_dir"))
                    .join(row.get::<String, &str>("team_name"))
                    .join(row.get::<String, &str>("title"));
                Ok(output.display().to_string())
            }
            Err(err) => {
                log::error!("couldn't get the project directory for pid {}: {}", pid, err);
                Ok("".to_string())
            }
        }
    }

    pub async fn get_file_info(&self, pid: i32, path: String) -> Result<UpdatedFile, ()> {
        let output = sqlx::query(
            "SELECT curr_hash, size, change_type, in_fs FROM file WHERE filepath = $1 AND pid = $2",
        )
        .bind(path.clone())
        .bind(pid)
        .fetch_one(self.pool)
        .await;
    
        match output {
            Ok(row) => {
                let change = match row.get::<i32, &str>("change_type") {
                    1 => ChangeType::Create,
                    2 => ChangeType::Update,
                    3 => ChangeType::Delete,
                    _ => ChangeType::NoChange,
                };
                let in_fs = if row.get::<i32, &str>("in_fs") > 0 { true } else { false };
                let owo: UpdatedFile = UpdatedFile {
                    path: path,
                    hash: row.get::<String, &str>("curr_hash").to_string(),
                    size: row.get::<i64, &str>("size"),
                    change: change,
                    in_fs: in_fs
                };
    
                Ok(owo)
            }
            Err(err) => {
                log::error!("couldn't get the file information for {} in project {}: {}", path, pid, err);
                Err(())
            }
        }
    }

    pub async fn get_basehash(&self, pid: i32, path: String) -> Result<String, ()> {
        let result = sqlx::query(
            "SELECT base_hash FROM file WHERE
            pid = $1 AND filepath = $2 LIMIT 1
            ",
        )
        .bind(pid)
        .bind(path)
        .fetch_one(self.pool)
        .await;
    
        match result {
            Ok(row) => Ok(row.get::<String, &str>("base_hash")),
            Err(err) => {
                log::error!("could not get base_hash: {}", err);
                Err(())
            }
        }
    }
    
    // TODO necessary to have in dal?
    pub async fn update_downloaded_file_entry(&self, pid: i32, path: String) -> Result<bool, ()> {
            let _ = sqlx::query(
            "
            UPDATE file SET
            base_hash = tracked_hash,
            curr_hash = tracked_hash,
            base_commitid = tracked_commitid,
            size = tracked_size,
            in_fs = 1,
            change_type = 0
            WHERE pid = $1 AND filepath = $2
            ",
        )
        .bind(pid)
        .bind(path)
        .execute(self.pool)
        .await;
        Ok(true)
    }

    pub async fn update_cache_setting(&self, new_cache: bool) -> Result<bool, ()> {
        let url = self.get_active_server().await.unwrap();
        match sqlx::query("UPDATE server SET cache_setting = $1 WHERE url = $2")
        .bind(if new_cache { 1 } else { 0 })
        .bind(url)
        .execute(self.pool)
        .await {
            Ok(_o) => {
                Ok(true)
            },
            Err(err) => {
                log::error!("could not set cache setting due to db error: {}", err);
                Ok(false)
            }
        }
    }

    pub async fn get_cache_setting(&self) -> Result<bool, ()> {
        let url = self.get_active_server().await.unwrap();
        match sqlx::query("SELECT cache_setting FROM server WHERE url = $1")
            .bind(url)
            .fetch_one(self.pool)
            .await {
                Ok(row) => {
                    let setting = row.get::<u32, &str>("cache_setting");
                    Ok(if setting == 1 { true } else { false })
                },
                Err(err) => {
                    log::error!("could not retrieve cache setting due to db error: {}", err);
                    Ok(false)
                }
        }
    }

    pub async fn clear_project_table(&self, url: String) -> Result<(), ()> {
        let _ = sqlx::query("DELETE from project WHERE url = $1")
            .bind(url.clone())
            .execute(self.pool)
            .await;
        Ok(())
    }

    pub async fn clear_file_table(&self) -> Result<(), ()> {
        let _ = sqlx::query("DELETE from file")
            .execute(self.pool)
            .await;
        Ok(())
    }

    pub async fn get_server_name(&self) -> Result<String, ()> {
        let output = sqlx::query("SELECT name FROM server WHERE active = 1")
        .fetch_one(self.pool)
        .await;

        match output {
            Ok(row) => Ok(row.get::<String, &str>("name")),
            Err(err) => {
                println!("couldnt get server name {}", err);
                Ok("glassyPDM".to_string())
            }
        }
    }

    pub async fn add_server(&self, url: String, clerk_pub_key: String, local_dir: String, name: String) -> Result<(), ()> {
        sqlx::query(
            "INSERT INTO server (url, clerk_publickey, local_dir, name, active, debug_url, debug_active) VALUES (?, ?, ?, ?, ?, ?, ?);"
        )
        .bind(url)
        .bind(clerk_pub_key)
        .bind(local_dir)
        .bind(name)
        .bind(1)
        .bind("http://localhost:5000")
        .bind(0)
        .execute(self.pool)
        .await.unwrap();

        Ok(())
    }

    pub async fn add_project(&self, pid: i32, title: String, team_name: String, init_commit: i32) -> Result<(), ()> {
        let server = self.get_active_server().await.unwrap();

        let _output = sqlx::query("INSERT INTO project(pid, url, title, team_name, base_commitid, remote_title) VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(pid, url) DO UPDATE SET remote_title = excluded.title")
            .bind(pid)
            .bind(server)
            .bind(title.clone())
            .bind(team_name)
            .bind(init_commit)
            .bind(title.clone())
            .execute(self.pool)
            .await;

        Ok(())
    }
} // end impl DataAcessLayer<'_>

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    /// initialize a db with a server and some projects
    async fn init_db(pool: &SqlitePool) {
        let dal = DataAccessLayer::new(&pool);

        // create server entry
        let _ = dal.add_server("url".to_string(), "key".to_string(), "owo/location".to_string(), "test server".to_string()).await;

        // create project entries
        let _ = dal.add_project(0, "project name".to_string(), "team name".to_string(), 0).await;
        let _ = dal.add_project(1, "project 2".to_string(), "team name".to_string(), 41).await;
        let _ = dal.add_project(14, "another project".to_string(), "team 2".to_string(), 2).await;

    }

    #[sqlx::test]
    async fn test_cache_setting(pool: SqlitePool) {
        let dal = DataAccessLayer::new(&pool);
        init_db(&pool).await;

        // verify initial settings
        let res = dal.get_cache_setting().await.unwrap();
        assert_eq!(res, false);

        // update settings
        let _ = dal.update_cache_setting(true).await.unwrap();

        // verify cache setting was updated
        let res = dal.get_cache_setting().await.unwrap();
        assert_eq!(res, true);
    }

    #[sqlx::test]
    async fn test_owo(pool: SqlitePool) {
        let dal = DataAccessLayer::new(&pool);
        init_db(&pool).await;

        
    }
}
