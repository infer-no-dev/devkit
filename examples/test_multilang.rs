//! Test script for multi-language blueprint functionality
//!
//! This example demonstrates the cross-language blueprint support

use devkit::blueprint::languages::{
    analyzers::create_analyzer_for_language, MultiLanguageAnalyzer,
};
use tempfile::TempDir;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Multi-Language Blueprint System");

    // Create a temporary directory with multiple languages
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    println!("📁 Creating test project at: {:?}", temp_path);

    // Create Rust project files
    let src_dir = temp_path.join("src");
    fs::create_dir_all(&src_dir).await?;

    let cargo_toml = temp_path.join("Cargo.toml");
    fs::write(
        &cargo_toml,
        r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
assert_cmd = "2.0"
"#,
    )
    .await?;

    let main_rs = src_dir.join("main.rs");
    fs::write(
        &main_rs,
        r#"
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    name: String,
    version: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
    };
    
    println!("Config: {:?}", config);
    Ok(())
}
"#,
    )
    .await?;

    // Create Python files
    let requirements_txt = temp_path.join("requirements.txt");
    fs::write(
        &requirements_txt,
        r#"
flask==2.3.0
requests>=2.25.0
pytest>=7.0.0
"#,
    )
    .await?;

    let main_py = temp_path.join("main.py");
    fs::write(
        &main_py,
        r#"
#!/usr/bin/env python3

import json
from flask import Flask, jsonify

app = Flask(__name__)

@app.route("/")
def hello():
    return jsonify({"message": "Hello from Python!", "version": "1.0.0"})

@app.route("/config")
def get_config():
    config = {
        "name": "test",
        "version": "1.0.0"
    }
    return jsonify(config)

if __name__ == "__main__":
    app.run(debug=True)
"#,
    )
    .await?;

    // Create JavaScript files
    let package_json = temp_path.join("package.json");
    fs::write(
        &package_json,
        r#"{
  "name": "test-project",
  "version": "1.0.0",
  "description": "Multi-language test project",
  "main": "index.js",
  "scripts": {
    "start": "node index.js",
    "test": "jest"
  },
  "dependencies": {
    "express": "^4.18.0",
    "axios": "^1.0.0"
  },
  "devDependencies": {
    "jest": "^29.0.0",
    "@types/node": "^18.0.0"
  }
}"#,
    )
    .await?;

    let index_js = temp_path.join("index.js");
    fs::write(
        &index_js,
        r#"
const express = require('express');
const axios = require('axios');

const app = express();
const PORT = process.env.PORT || 3000;

app.get('/', (req, res) => {
    res.json({ 
        message: 'Hello from JavaScript!', 
        version: '1.0.0',
        language: 'javascript'
    });
});

app.get('/config', (req, res) => {
    const config = {
        name: 'test',
        version: '1.0.0',
        language: 'javascript'
    };
    res.json(config);
});

app.listen(PORT, () => {
    console.log(`Server running on port ${PORT}`);
});
"#,
    )
    .await?;

    println!("✅ Test project created successfully");

    // Test language detection and analysis
    println!("\n🔍 Testing Language Detection...");

    let analyzer = MultiLanguageAnalyzer::new();
    let detected_languages = analyzer.detect_languages(temp_path).await?;

    println!("Detected languages: {:?}", detected_languages);

    // Test individual language analyzers
    println!("\n🔧 Testing Individual Analyzers...");

    for language in &detected_languages {
        println!("\n--- Analyzing {:?} ---", language);

        let lang_analyzer = create_analyzer_for_language(language);
        match lang_analyzer.analyze(temp_path).await {
            Ok(module) => {
                println!("✅ {:?} analysis successful:", language);
                println!("   Language: {:?}", module.language);
                println!("   Entry points: {:?}", module.entry_points);
                println!("   Dependencies: {} found", module.dependencies.len());
                println!("   Build tool: {}", module.build_config.build_tool);
                println!("   Test framework: {}", module.test_config.test_framework);

                // Show first few dependencies
                for (i, dep) in module.dependencies.iter().take(3).enumerate() {
                    println!(
                        "   Dep {}: {} v{} ({})",
                        i + 1,
                        dep.name,
                        dep.version,
                        dep.purpose
                    );
                }
                if module.dependencies.len() > 3 {
                    println!("   ... and {} more", module.dependencies.len() - 3);
                }
            }
            Err(e) => {
                println!("❌ {:?} analysis failed: {}", language, e);
            }
        }
    }

    // Test multi-language project analysis
    println!("\n🌐 Testing Multi-Language Project Analysis...");

    let blueprint = analyzer.analyze_project(temp_path).await?;
    println!("✅ Analyzed multi-language project:");
    println!("   Primary language: {:?}", blueprint.primary_language);
    println!(
        "   Secondary languages: {:?}",
        blueprint.secondary_languages
    );
    println!("   Language modules: {}", blueprint.language_modules.len());

    for (lang, module) in &blueprint.language_modules {
        println!(
            "   - {:?}: {} dependencies, {} entry points",
            lang,
            module.dependencies.len(),
            module.entry_points.len()
        );
    }

    // Test interface extraction
    println!("\n🔌 Testing Cross-Language Interface Detection...");
    println!(
        "✅ Detected {} cross-language interfaces",
        blueprint.inter_language_interfaces.len()
    );
    for interface in &blueprint.inter_language_interfaces {
        println!(
            "   - {:?} -> {:?}: {:?}",
            interface.source_language, interface.target_language, interface.binding_type
        );
    }

    // Test build orchestration analysis
    println!("\n🏗️  Testing Build Orchestration...");
    println!(
        "✅ Build orchestration: {}",
        blueprint.build_orchestration.orchestration_tool
    );
    println!(
        "   - Build order: {:?}",
        blueprint.build_orchestration.build_order
    );
    println!(
        "   - Parallel builds: {}",
        blueprint.build_orchestration.parallel_builds
    );

    // Test deployment strategy
    println!("\n🚀 Testing Deployment Strategy...");
    println!(
        "✅ Deployment platform: {}",
        blueprint.deployment_strategy.orchestration.platform
    );

    println!("\n🎉 Multi-Language Blueprint System Test Completed Successfully!");

    Ok(())
}
