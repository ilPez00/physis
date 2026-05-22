use physis::ai::provider::ProviderCascade;
use physis::ai::agent::{run_agent, AgentConfig};
use physis::ai::tools::ToolRegistry;
use physis::{OntologyLoader, PhysisConfig, DreamEngine, Goal};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use axum::{extract::State, routing::get, Json, Router};
use dotenvy::dotenv;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct AppState {
    entities: Vec<String>, // Simplification for UI display
    relationships: Vec<(String, String, String)>,
    transcripts: Vec<String>,
    dreams: Vec<String>,
}

type SharedState = Arc<Mutex<AppState>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();

    let shared_state: SharedState = Arc::new(Mutex::new(AppState::default()));
    let config = PhysisConfig::default();
    let ontology = OntologyLoader::load_all(&config);
    let mut mapper = physis::OntologyMapper::new(ontology);
    let mut dream_engine = DreamEngine::new(mapper.trie.clone());

    // Start Web Server
    let app_state_clone = shared_state.clone();
    tokio::spawn(async move {
        let app = Router::new()
            .route("/api/data", get(get_data))
            .route("/", get(index));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        println!("Physis Voice UI: http://localhost:3000");
        axum::serve(listener, app.with_state(app_state_clone)).await.unwrap();
    });

    let cascade = ProviderCascade::from_env();
    let tools = ToolRegistry::new();

    loop {
        let wav_path = "chunk.wav";
        let _ = Command::new("arecord")
            .args(&["-d", "10", "-f", "S16_LE", "-r", "16000", "-t", "wav", wav_path])
            .status();

        if let Ok(transcript) = cascade.transcribe(wav_path).await {
            if transcript.trim().is_empty() { continue; }
            
            println!("Transcript: {}", transcript);
            
            let mut state = shared_state.lock().unwrap();
            state.transcripts.push(transcript.clone());

            // Extract via AI
            let agent_config = AgentConfig {
                system_prompt: "Extract entities (CSV) and relationships (Source|Predicate|Target). Be very brief.".into(),
                ..Default::default()
            };
            
            if let Ok(output) = run_agent(&cascade, &tools, &agent_config, &[], &transcript, None, "DATA").await {
                // Parse entities/relationships from text (Simplified for demo)
                for line in output.text.lines() {
                    if line.contains('|') {
                        let parts: Vec<&str> = line.split('|').collect();
                        if parts.len() == 3 {
                            state.relationships.push((parts[0].trim().into(), parts[1].trim().into(), parts[2].trim().into()));
                            state.entities.push(parts[0].trim().into());
                            state.entities.push(parts[2].trim().into());
                        }
                    }
                }
            }
            
            // Dream
            let goals = vec![Goal::new(&transcript, "voice_stream")];
            let dreams = dream_engine.generate_dreams(&goals, 2);
            for d in dreams {
                state.dreams.push(format!("{} -> {}", d.id, d.description));
            }
        }
    }
}

async fn get_data(State(state): State<SharedState>) -> Json<AppState> {
    Json(state.lock().unwrap().clone())
}

async fn index() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("index_v2.html"))
}
