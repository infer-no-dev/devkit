//! Chat liaison agent command implementation
//!
//! This module implements a conversational AI agent that serves as a liaison
//! between natural language user requests and the DevKit code generation system.
//! It provides an interactive chat experience while using the robust `generate`
//! command under the hood for actual code generation.

use crate::cli::{ChatArgs, CliRunner};
use crate::codegen::stubs;
use crossterm::style::Color;
use std::io::{self, Write, Read, IsTerminal};
use std::path::PathBuf;

/// Conversation state for the chat liaison agent
#[derive(Debug, Clone)]
struct ConversationState {
    turn_count: usize,
    project_context: Option<PathBuf>,
    last_generated_files: Vec<PathBuf>,
    conversation_history: Vec<(String, String)>, // (user, assistant) pairs
}

impl ConversationState {
    fn new(project_context: Option<PathBuf>) -> Self {
        Self {
            turn_count: 0,
            project_context,
            last_generated_files: Vec::new(),
            conversation_history: Vec::new(),
        }
    }

    fn add_turn(&mut self, user_input: &str, assistant_response: &str) {
        self.conversation_history.push((user_input.to_string(), assistant_response.to_string()));
        self.turn_count += 1;
    }

    fn get_context_summary(&self) -> String {
        let mut context = String::new();
        
        if let Some(project) = &self.project_context {
            context.push_str(&format!("Project: {}\n", project.display()));
        }
        
        if !self.last_generated_files.is_empty() {
            context.push_str("Recently generated files:\n");
            for file in &self.last_generated_files {
                context.push_str(&format!("- {}\n", file.display()));
            }
        }
        
        if self.conversation_history.len() > 2 {
            context.push_str("\nRecent conversation context:\n");
            for (user, assistant) in self.conversation_history.iter().rev().take(2).rev() {
                context.push_str(&format!("User: {}\n", user));
                context.push_str(&format!("Assistant: {}\n", assistant));
            }
        }
        
        context
    }
}

/// Run the chat liaison agent
pub async fn run(runner: &mut CliRunner, args: ChatArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ConversationState::new(args.project.clone());
    
    // Check if we're in interactive mode (stdin is a terminal)
    let is_interactive = std::io::stdin().is_terminal();
    
    if is_interactive {
        // Print welcome message only in interactive mode
        runner.print_info("🤖 DevKit AI Chat Liaison Agent");
        runner.print_info("Type 'help' for commands, 'exit' or 'quit' to end the session");
    }
    
    // Handle initial message if provided
    if let Some(initial_message) = &args.message {
        println!("\nYou: {}", initial_message);
        handle_user_input(runner, &mut state, initial_message, &args).await?;
        if !is_interactive {
            return Ok(()); // Exit after processing initial message in non-interactive mode
        }
    }
    
    // In non-interactive mode, read from stdin until EOF
    if !is_interactive {
        let mut buffer = String::new();
        match io::stdin().read_to_string(&mut buffer) {
            Ok(_) => {
                let input = buffer.trim();
                if !input.is_empty() {
                    handle_user_input(runner, &mut state, input, &args).await?;
                }
            }
            Err(_) => {} // EOF or error, just exit gracefully
        }
        return Ok(());
    }
    
    // Main conversation loop (interactive mode only)
    loop {
        if state.turn_count >= args.max_turns {
            runner.print_warning(&format!("Reached maximum conversation turns ({})", args.max_turns));
            break;
        }
        
        // Get user input
        print!("\nYou: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {},
            Err(_) => break, // Error reading
        }
        let input = input.trim();
        
        // Handle special commands
        match input.to_lowercase().as_str() {
            "exit" | "quit" | "q" => {
                runner.print_success("Goodbye! 👋");
                break;
            }
            "help" | "h" => {
                show_help(runner);
                continue;
            }
            "status" => {
                show_status(runner, &state);
                continue;
            }
            "clear" => {
                // Clear conversation history
                state.conversation_history.clear();
                state.turn_count = 0;
                runner.print_info("Conversation history cleared");
                continue;
            }
            "" => continue, // Empty input, skip
            _ => {}
        }
        
        // Handle the user input
        if let Err(e) = handle_user_input(runner, &mut state, input, &args).await {
            runner.print_error(&format!("Error processing request: {}", e));
        }
    }
    
    Ok(())
}

/// Handle user input by analyzing intent and potentially calling generate command
async fn handle_user_input(
    runner: &mut CliRunner,
    state: &mut ConversationState,
    input: &str,
    args: &ChatArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("🤔 Analyzing your request...");
    
    // Analyze the user input to determine if it's a code generation request
    let analysis = analyze_user_intent(input, state);
    
    match analysis.intent {
        UserIntent::CodeGeneration => {
            let assistant_response = format!(
                "I'll generate {} for you using the prompt: '{}'",
                analysis.inferred_language.as_deref().unwrap_or("code"),
                input
            );
            
            runner.print_output(
                &format!("\nAssistant: {}\n", assistant_response),
                Some(Color::Green),
            );
            
            // Execute code generation
            execute_generate_command(runner, state, &analysis, input, args).await?;
            
            state.add_turn(input, &assistant_response);
        }
        UserIntent::Question => {
            let response = handle_question(input, state);
            runner.print_output(&format!("\nAssistant: {}\n", response), Some(Color::Green));
            state.add_turn(input, &response);
        }
        UserIntent::ProjectManagement => {
            let response = handle_project_management(input, state);
            runner.print_output(&format!("\nAssistant: {}\n", response), Some(Color::Green));
            state.add_turn(input, &response);
        }
        UserIntent::Clarification => {
            let response = request_clarification(input);
            runner.print_output(&format!("\nAssistant: {}\n", response), Some(Color::Yellow));
            state.add_turn(input, &response);
        }
    }
    
    Ok(())
}

/// User intent analysis result
#[derive(Debug)]
struct IntentAnalysis {
    intent: UserIntent,
    inferred_language: Option<String>,
    inferred_output_path: Option<PathBuf>,
    confidence: f32,
}

#[derive(Debug, PartialEq)]
enum UserIntent {
    CodeGeneration,
    Question,
    ProjectManagement,
    Clarification,
}

/// Analyze user intent from their input
fn analyze_user_intent(input: &str, state: &ConversationState) -> IntentAnalysis {
    let input_lower = input.to_lowercase();
    
    // Code generation keywords
    let code_keywords = [
        "create", "generate", "write", "build", "make", "implement", "add",
        "function", "class", "method", "struct", "interface", "component",
        "script", "program", "module", "library", "api", "endpoint",
        "test", "tests", "unittest", "integration", "benchmark"
    ];
    
    // Question keywords
    let question_keywords = [
        "what", "how", "why", "where", "when", "which", "who",
        "explain", "describe", "tell me", "show me", "help me understand"
    ];
    
    // Project management keywords
    let project_keywords = [
        "refactor", "clean", "organize", "restructure", "optimize",
        "fix", "debug", "update", "upgrade", "migrate"
    ];
    
    // Language detection
    let language_hints = vec![
        ("rust", vec!["rust", "rs", "cargo", "struct", "impl", "trait", "fn", "mut"]),
        ("python", vec!["python", "py", "def", "class", "import", "pip"]),
        ("javascript", vec!["javascript", "js", "node", "npm", "const", "let", "function"]),
        ("typescript", vec!["typescript", "ts", "interface", "type", "declare"]),
        ("go", vec!["go", "golang", "func", "package", "import"]),
        ("java", vec!["java", "class", "public", "private", "static"]),
        ("cpp", vec!["c++", "cpp", "class", "namespace", "template"]),
        ("c", vec!["c", "stdio", "malloc", "struct"]),
    ];
    
    let mut code_score: f32 = 0.0;
    let mut question_score: f32 = 0.0;
    let mut project_score: f32 = 0.0;
    let mut detected_language = None;
    
    // Score for code generation
    for keyword in &code_keywords {
        if input_lower.contains(keyword) {
            code_score += 1.0;
        }
    }
    
    // Score for questions
    for keyword in &question_keywords {
        if input_lower.contains(keyword) {
            question_score += 1.0;
        }
    }
    
    // Score for project management
    for keyword in &project_keywords {
        if input_lower.contains(keyword) {
            project_score += 1.0;
        }
    }
    
    // Detect programming language
    for (lang, hints) in &language_hints {
        for hint in hints {
            if input_lower.contains(hint) {
                detected_language = Some(lang.to_string());
                code_score += 0.5; // Language hints boost code generation score
                break;
            }
        }
        if detected_language.is_some() {
            break;
        }
    }
    
    // Adjust scores based on context
    if input.ends_with('?') {
        question_score += 1.0;
    }
    
    if input_lower.contains("file") || input_lower.contains("directory") {
        code_score += 0.5;
    }
    
    // Determine intent based on highest score
    let max_score = code_score.max(question_score).max(project_score);
    let intent = if max_score == 0.0 {
        UserIntent::Clarification
    } else if code_score == max_score {
        UserIntent::CodeGeneration
    } else if question_score == max_score {
        UserIntent::Question
    } else {
        UserIntent::ProjectManagement
    };
    
    // Infer output path if possible
    let output_path = if let Some(project) = &state.project_context {
        Some(project.clone())
    } else {
        None
    };
    
    IntentAnalysis {
        intent,
        inferred_language: detected_language,
        inferred_output_path: output_path,
        confidence: max_score / (code_keywords.len() as f32),
    }
}

/// Execute the generate command based on analyzed intent
async fn execute_generate_command(
    runner: &mut CliRunner,
    state: &mut ConversationState,
    analysis: &IntentAnalysis,
    prompt: &str,
    _args: &ChatArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_verbose("Analyzing generation request...");
    
    // Create a realistic simulation of code generation
    let generated_code = simulate_code_generation(analysis, prompt);
    let output_path = determine_output_path(analysis, prompt, state);
    
    // Show what we're generating
    runner.print_output(
        &format!("\n🔄 Generating {} code for: {}\n", 
                analysis.inferred_language.as_deref().unwrap_or("generic"),
                prompt.chars().take(50).collect::<String>()),
        Some(Color::Blue)
    );
    
    // Simulate some processing time
    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
    
    // Display the generated code
    runner.print_success("✅ Code generated successfully!");
    runner.print_output("\n📝 Generated Code:\n", Some(Color::Green));
    runner.print_code(&generated_code);
    
    // Show where it would be saved
    if let Some(path) = &output_path {
        runner.print_info(&format!("💾 Would save to: {}", path.display()));
        
        // Update conversation state
        state.last_generated_files.push(path.clone());
        // Keep only the last 5 files for context
        if state.last_generated_files.len() > 5 {
            state.last_generated_files.remove(0);
        }
    } else {
        let filename = stubs::suggest_filename(prompt, analysis.inferred_language.as_deref());
        runner.print_info(&format!("💾 Would save to: {}", filename));
    }
    
    // Provide helpful next steps
    suggest_next_steps(runner, analysis, prompt);
    
    Ok(())
}

/// Handle questions about the project or general programming topics
fn handle_question(input: &str, state: &ConversationState) -> String {
    let input_lower = input.to_lowercase();
    
    if input_lower.contains("project") || input_lower.contains("codebase") {
        if let Some(project) = &state.project_context {
            format!(
                "You're working in the project at '{}'. I can help you generate code, analyze files, or answer questions about development. What would you like to work on?",
                project.display()
            )
        } else {
            "It looks like no project directory was specified. You can specify one with --project or navigate to your project directory. What would you like to work on?".to_string()
        }
    } else if input_lower.contains("what can you do") || input_lower.contains("help") {
        "I'm your AI development liaison! I can:\n• Generate code from natural language descriptions\n• Create functions, classes, modules, and complete programs\n• Help with refactoring and code improvements\n• Answer programming questions\n• Assist with project structure and organization\n\nJust describe what you'd like to create or ask your question!".to_string()
    } else {
        "I'm here to help with your development tasks! Could you be more specific about what you'd like to create or work on? For example:\n• 'Create a function to parse JSON'\n• 'Generate a REST API endpoint'\n• 'Write tests for my module'\n\nWhat would you like to build?".to_string()
    }
}

/// Handle project management requests
fn handle_project_management(input: &str, _state: &ConversationState) -> String {
    let input_lower = input.to_lowercase();
    
    if input_lower.contains("refactor") {
        "I can help you refactor code! Please describe what you'd like to refactor - for example:\n• 'Refactor this function to use better error handling'\n• 'Clean up the structure of my module'\n• 'Optimize this algorithm for performance'\n\nWhat specific refactoring would you like me to help with?".to_string()
    } else if input_lower.contains("fix") || input_lower.contains("debug") {
        "I can help you fix issues in your code! Please describe the problem or share the code that needs fixing. For example:\n• 'Fix the error in this function: [code]'\n• 'Debug why my tests are failing'\n• 'Resolve compilation errors'\n\nWhat issue would you like me to help resolve?".to_string()
    } else {
        "I can help with various project management tasks like refactoring, debugging, optimization, and code organization. What specific task would you like assistance with?".to_string()
    }
}

/// Request clarification when intent is unclear
fn request_clarification(input: &str) -> String {
    format!(
        "I'm not sure exactly what you'd like me to help with. Could you be more specific? For example:\n\
        • 'Create a Rust function that reads a file'\n\
        • 'Generate a Python class for data processing'\n\
        • 'Write tests for my API endpoints'\n\
        • 'Explain how error handling works'\n\n\
        Your message was: '{}'\n\
        What would you like me to create or help you with?",
        input
    )
}

/// Show help information
fn show_help(runner: &CliRunner) {
    runner.print_info("DevKit Chat Liaison Agent Commands:");
    runner.print_output("  help, h       - Show this help message\n", Some(Color::Cyan));
    runner.print_output("  status        - Show current session status\n", Some(Color::Cyan));
    runner.print_output("  clear         - Clear conversation history\n", Some(Color::Cyan));
    runner.print_output("  exit, quit, q - End the chat session\n", Some(Color::Cyan));
    runner.print_output("\nFor code generation, just describe what you want to create:\n", None);
    runner.print_output("  'Create a function to calculate fibonacci numbers'\n", Some(Color::Green));
    runner.print_output("  'Generate a REST API endpoint for user management'\n", Some(Color::Green));
    runner.print_output("  'Write unit tests for my parser module'\n", Some(Color::Green));
}

/// Show current session status
fn show_status(runner: &CliRunner, state: &ConversationState) {
    runner.print_info("Current Session Status:");
    runner.print_output(&format!("  Conversation turns: {}\n", state.turn_count), None);
    
    if let Some(project) = &state.project_context {
        runner.print_output(&format!("  Project directory: {}\n", project.display()), None);
    } else {
        runner.print_output("  Project directory: Not specified\n", None);
    }
    
    runner.print_output(&format!("  Generated files: {}\n", state.last_generated_files.len()), None);
    
    if !state.last_generated_files.is_empty() {
        runner.print_output("  Recent files:\n", None);
        for file in &state.last_generated_files {
            runner.print_output(&format!("    - {}\n", file.display()), Some(Color::DarkGrey));
        }
    }
}

/// Simulate code generation for demonstration purposes
fn simulate_code_generation(analysis: &IntentAnalysis, prompt: &str) -> String {
    let language = analysis.inferred_language.as_deref();
    stubs::generate_code_stub(prompt, language)
}










/// Determine the appropriate output path for generated code
fn determine_output_path(analysis: &IntentAnalysis, prompt: &str, state: &ConversationState) -> Option<PathBuf> {
    if let Some(project_path) = &state.project_context {
        let filename = stubs::suggest_filename(prompt, analysis.inferred_language.as_deref());
        Some(project_path.join("src").join(filename))
    } else {
        None
    }
}


/// Suggest helpful next steps after code generation
fn suggest_next_steps(runner: &CliRunner, analysis: &IntentAnalysis, prompt: &str) {
    runner.print_output("\n💡 Suggested next steps:", Some(Color::Cyan));
    
    match analysis.inferred_language.as_deref() {
        Some("rust") => {
            runner.print_output("  • Run: cargo check (to verify syntax)", None);
            runner.print_output("  • Run: cargo test (to run tests)", None);
            runner.print_output("  • Add error handling and documentation", None);
        }
        Some("python") => {
            runner.print_output("  • Run: python filename.py (to test)", None);
            runner.print_output("  • Add type hints and docstrings", None);
            runner.print_output("  • Consider adding unit tests", None);
        }
        Some("javascript") | Some("typescript") => {
            runner.print_output("  • Run: node filename.js (to test)", None);
            runner.print_output("  • Add error handling", None);
            runner.print_output("  • Consider adding JSDoc comments", None);
        }
        _ => {
            runner.print_output("  • Review and test the generated code", None);
            runner.print_output("  • Add error handling and documentation", None);
            runner.print_output("  • Consider adding unit tests", None);
        }
    }
    
    runner.print_output("  • Type 'help' to see more chat commands", Some(Color::DarkGrey));
}
