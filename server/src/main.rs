use anyhow::Result;
use postgrest::Postgrest;
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, sync::broadcast};
use futures::{StreamExt, TryStreamExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Event {
    id: String,
    status: String,
    created_at: String,
}

struct SupabaseClient {
    client: Postgrest,
    realtime: Option<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
}

impl SupabaseClient {
    fn new() -> Result<Self> {
        dotenv::dotenv().ok();
        
        let supabase_url = std::env::var("SUPABASE_URL")
            .expect("SUPABASE_URL must be set");
        let supabase_key = std::env::var("SUPABASE_KEY")
            .expect("SUPABASE_KEY must be set");

        let client = Postgrest::new(supabase_url)
            .insert_header("apikey", &supabase_key)
            .insert_header("Authorization", format!("Bearer {}", supabase_key));

        Ok(Self { 
            client,
            realtime: None,
        })
    }

    async fn connect_realtime(&mut self) -> Result<()> {
        let ws_url = format!("{}/realtime/v1/websocket", 
            std::env::var("SUPABASE_URL")?
                .replace("http", "ws"));

        let (ws_stream, _) = tokio_tungstenite::connect_async(ws_url).await?;
        self.realtime = Some(ws_stream);
        Ok(())
    }

    async fn subscribe_to_events(&mut self) -> Result<broadcast::Receiver<Event>> {
        let (tx, rx) = broadcast::channel(100);
        
        if let Some(ws_stream) = &mut self.realtime {
            let mut stream = ws_stream.try_filter(|msg| {
                future::ready(msg.is_text() && msg.to_text().unwrap().contains("events"))
            });

            tokio::spawn(async move {
                while let Ok(Some(msg)) = stream.next().await {
                    if let Ok(event) = serde_json::from_str::<Event>(msg.to_text().unwrap()) {
                        let _ = tx.send(event);
                    }
                }
            });
        }

        Ok(rx)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut supabase = SupabaseClient::new()?;
    supabase.connect_realtime().await?;
    
    let listener = TcpListener::bind("127.0.0.1:8888").await?;
    println!("Server listening on port 8888");

    let event_rx = supabase.subscribe_to_events().await?;

    loop {
        tokio::select! {
            Ok((socket, addr)) = listener.accept() => {
                println!("New connection from: {}", addr);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(socket).await {
                        eprintln!("Connection error: {}", e);
                    }
                });
            }
            
            Ok(event) = event_rx.recv() => {
                if let Err(e) = handle_event(&event).await {
                    eprintln!("Event handling error: {}", e);
                }
            }
        }
    }
}

async fn handle_connection(socket: tokio::net::TcpStream) -> Result<()> {
    // Implement socket handling logic
    Ok(())
}

async fn handle_event(event: &Event) -> Result<()> {
    println!("Received realtime event: {:?}", event);
    Ok(())
}
