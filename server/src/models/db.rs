

// use anyhow::Result;
// use sqlx::{Pool, Postgres};
// use serde_json::Value;

// impl Database {

//     /// Create
//     pub async fn insert_record(&self, table: &str, data: &Value) -> Result<i64> {
//         let columns: Vec<&str> = data.as_object()
//             .unwrap()
//             .keys()
//             .map(|s| s.as_str())
//             .collect();
        
//         let values: Vec<&Value> = data.as_object()
//             .unwrap()
//             .values()
//             .collect();
        
//         let placeholders: Vec<String> = (1..=values.len())
//             .map(|i| format!("${}", i))
//             .collect();

//         let query = format!(
//             "INSERT INTO {} ({}) VALUES ({}) RETURNING id",
//             table,
//             columns.join(", "),
//             placeholders.join(", ")
//         );

//         let query = sqlx::query(&query)
//             .bind_all(values)
//             .fetch_one(&self.pool)
//             .await?;

//         Ok(query.get("id"))
//     }

//     // Update
//     pub async fn update_record(&self, table: &str, id: i64, data: &Value) -> Result<()> {
//         let updates: Vec<String> = data.as_object()
//             .unwrap()
//             .keys()
//             .enumerate()
//             .map(|(i, k)| format!("{} = ${}", k, i + 1))
//             .collect();

//         let query = format!(
//             "UPDATE {} SET {} WHERE id = ${}",
//             table,
//             updates.join(", "),
//             updates.len() + 1
//         );

//         sqlx::query(&query)
//             .bind_all(data.as_object().unwrap().values())
//             .bind(id)
//             .execute(&self.pool)
//             .await?;

//         Ok(())
//     }

//     // Delete
//     pub async fn delete_record(&self, table: &str, id: i64) -> Result<()> {
//         sqlx::query(&format!("DELETE FROM {} WHERE id = $1", table))
//             .bind(id)
//             .execute(&self.pool)
//             .await?;
            
//         Ok(())
//     }
// }