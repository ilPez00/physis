use clap::Parser;

use physis::cli::{Cli, Commands, PhysisApp};
use physis::config::PhysisConfig;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = PhysisConfig::default();
    let mut app = PhysisApp::new(config);

    match cli.command {
        Commands::Scan { dir, format } => {
            let output = app.run_scan(&dir, &format);
            println!("{output}");
        }
        Commands::Query { query, max_results } => {
            let results = app.run_query(&query, max_results);
            if results.is_empty() {
                println!("No results found for: {query}");
            } else {
                for path in results {
                    println!("  {}", path.join(" → "));
                }
            }
        }
        Commands::Dream { count } => {
            let dreams = app.run_dream(count);
            for dream in &dreams {
                println!("[{}] {} — {}",
                    dream.dream_type.as_str(),
                    dream.id,
                    dream.description);
                println!("  variation: {}", dream.variation.join(" → "));
                println!();
            }
        }
        Commands::Evaluate { id, grade } => {
            if app.run_evaluate(&id, grade) {
                println!("Dream {id} evaluated with grade {grade}");
            } else {
                println!("Dream {id} not found");
            }
        }
        #[cfg(feature = "mcp")]
        Commands::Serve => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let config = PhysisConfig::default();
                physis::mcp::start_mcp_server(config).await
            })?;
        }
        Commands::Watch { dirs } => {
            let output = app.run_watch(dirs);
            println!("{output}");
        }
        Commands::Stats => {
            let output = app.run_stats();
            println!("{output}");
        }
        Commands::Config => {
            let config_str = serde_json::to_string_pretty(&PhysisConfig::default())?;
            println!("{config_str}");
        }
    }

    Ok(())
}
