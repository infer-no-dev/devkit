//! Demo showcasing enhanced UI components with error handling and progress indicators.

use devkit::{
    agents::{AgentStatus, TaskPriority},
    error::DevKitError,
    ui::{
        enhanced_panels::{
            AlertLevel, EnhancedAgentInfo, EnhancedPanel, EnhancedPanelManager,
            LogEntry, LogLevel, OutputLine, OutputType, PanelContent, PanelStyleConfig,
            ResourceUsage, StatusAlert, SystemMetrics, AgentSummary,
        },
        error_handler::{UIErrorHandler, ErrorSeverity},
        notifications::{Notification, NotificationType},
        progress::{ProgressManager, ProgressStyle},
        themes::{Theme, ThemeManager},
    },
};
use std::{collections::HashMap, time::{Duration, Instant}};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 DevKit Enhanced UI Components Demo");

    // Setup notification system
    let (notification_tx, mut notification_rx) = mpsc::unbounded_channel();
    
    // Initialize enhanced components
    let mut error_handler = UIErrorHandler::new(notification_tx.clone());
    let progress_manager = ProgressManager::new();
    let mut enhanced_panel_manager = EnhancedPanelManager::new();
    let theme_manager = ThemeManager::new();
    let theme = theme_manager.current_theme();

    // Demo error handling
    println!("📋 Testing error handling...");
    let test_error = DevKitError::ValidationError {
        field: "input".to_string(),
        message: "Value must be between 1 and 100".to_string(),
    };
    let recovery_strategy = error_handler.handle_error(test_error).await;
    println!("✅ Error handling strategy: {:?}", recovery_strategy);

    // Receive notifications
    if let Ok(notification) = notification_rx.try_recv() {
        println!("📢 Received notification: {}", notification.title);
    }

    // Demo progress management
    println!("📊 Testing progress indicators...");
    let progress_tracker = progress_manager.start_operation(
        "Code Analysis".to_string(),
        Some("Analyzing codebase structure...".to_string()),
        ProgressStyle::Steps,
        Some(Duration::from_secs(30)),
        vec![
            "Scanning files".to_string(),
            "Parsing syntax".to_string(),
            "Building symbol table".to_string(),
            "Generating report".to_string(),
        ],
    ).await;

    // Simulate progress updates
    for step in 0..4 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        progress_tracker.update_step(step, 1.0, Some(format!("Completed step {}", step + 1)));
    }
    progress_tracker.complete(Some("Analysis completed successfully!".to_string()));

    // Demo enhanced panels
    println!("🎨 Setting up enhanced panels...");
    
    // Agent status panel
    let agent_panel = create_agent_status_panel();
    enhanced_panel_manager.add_panel(agent_panel);

    // Output panel with colored lines
    let output_panel = create_output_panel();
    enhanced_panel_manager.add_panel(output_panel);

    // System status panel
    let status_panel = create_system_status_panel();
    enhanced_panel_manager.add_panel(status_panel);

    // Logs panel
    let logs_panel = create_logs_panel();
    enhanced_panel_manager.add_panel(logs_panel);

    // Test panel operations
    println!("⚙️  Testing panel operations...");
    enhanced_panel_manager.focus_panel("agents").expect("Failed to focus agents panel");
    println!("Focused panel: {:?}", enhanced_panel_manager.get_focused_panel());

    enhanced_panel_manager.toggle_panel_visibility("output")?;
    println!("Toggled output panel visibility");

    // Test error scenarios
    println!("🧪 Testing error scenarios...");
    let errors = vec![
        DevKitError::AI("Model request timeout".to_string()),
        DevKitError::Context("Failed to parse syntax tree".to_string()),
        DevKitError::Http("Network unreachable".to_string()),
        DevKitError::IO(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Access denied"
        )),
    ];

    for error in errors {
        let strategy = error_handler.handle_error(error).await;
        if let Ok(notification) = notification_rx.try_recv() {
            println!("📢 Error notification: {} - {}", notification.title, notification.message);
        }
    }

    // Display summary
    let active_operations = progress_manager.get_active_operations().await;
    let recent_errors = error_handler.get_recent_errors(5);
    
    println!("\n📈 Demo Summary:");
    println!("Active operations: {}", active_operations.len());
    println!("Recent errors: {}", recent_errors.len());
    println!("Enhanced panels: {}", 4);

    println!("✨ Enhanced UI components demo completed successfully!");
    
    Ok(())
}

fn create_agent_status_panel() -> EnhancedPanel {
    let agents = vec![
        EnhancedAgentInfo {
            name: "CodeGen Agent".to_string(),
            status: AgentStatus::Processing { task_id: "task_001".to_string() },
            current_task: Some("Generating React component".to_string()),
            priority: Some(TaskPriority::High),
            progress: Some(0.75),
            last_activity: Instant::now(),
            error_count: 0,
            success_count: 12,
            estimated_completion: Some(Instant::now() + Duration::from_secs(30)),
            resource_usage: ResourceUsage {
                cpu_percent: 45.2,
                memory_mb: 256,
                network_bytes: 1024 * 1024,
                disk_io: 512 * 1024,
            },
        },
        EnhancedAgentInfo {
            name: "Analysis Agent".to_string(),
            status: AgentStatus::Idle,
            current_task: None,
            priority: None,
            progress: None,
            last_activity: Instant::now() - Duration::from_secs(120),
            error_count: 1,
            success_count: 8,
            estimated_completion: None,
            resource_usage: ResourceUsage {
                cpu_percent: 12.1,
                memory_mb: 128,
                network_bytes: 0,
                disk_io: 0,
            },
        },
    ];

    let summary = AgentSummary {
        total_agents: 2,
        active_agents: 1,
        idle_agents: 1,
        failed_agents: 0,
        total_tasks: 21,
        completed_tasks: 20,
        avg_response_time: Duration::from_millis(450),
    };

    EnhancedPanel {
        id: "agents".to_string(),
        title: "AI Agents".to_string(),
        content: PanelContent::AgentStatus { agents, summary },
        style_config: PanelStyleConfig::default(),
        scroll_position: 0,
        max_items: None,
        auto_scroll: false,
        visible: true,
        focused: false,
        last_updated: Instant::now(),
        error_count: 1,
        warning_count: 0,
    }
}

fn create_output_panel() -> EnhancedPanel {
    let lines = vec![
        OutputLine {
            content: "Starting code generation...".to_string(),
            line_type: OutputType::Command,
            timestamp: Instant::now() - Duration::from_secs(10),
            metadata: None,
        },
        OutputLine {
            content: "Successfully generated component structure".to_string(),
            line_type: OutputType::Success,
            timestamp: Instant::now() - Duration::from_secs(8),
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("component".to_string(), "UserProfile".to_string());
                meta
            }),
        },
        OutputLine {
            content: "Warning: TypeScript definitions missing for some props".to_string(),
            line_type: OutputType::Warning,
            timestamp: Instant::now() - Duration::from_secs(5),
            metadata: None,
        },
        OutputLine {
            content: "Error: Failed to import required dependency".to_string(),
            line_type: OutputType::Error,
            timestamp: Instant::now() - Duration::from_secs(2),
            metadata: None,
        },
        OutputLine {
            content: "Generated React component with hooks and TypeScript support".to_string(),
            line_type: OutputType::Result,
            timestamp: Instant::now(),
            metadata: None,
        },
    ];

    EnhancedPanel {
        id: "output".to_string(),
        title: "Command Output".to_string(),
        content: PanelContent::Output { lines, filter: None },
        style_config: PanelStyleConfig::default(),
        scroll_position: 0,
        max_items: Some(100),
        auto_scroll: true,
        visible: true,
        focused: false,
        last_updated: Instant::now(),
        error_count: 1,
        warning_count: 1,
    }
}

fn create_system_status_panel() -> EnhancedPanel {
    let metrics = SystemMetrics {
        cpu_usage: 0.35,
        memory_usage: 0.42,
        disk_usage: 0.78,
        network_activity: 1024.0 * 1024.0 * 2.5, // 2.5 MB/s
        active_connections: 15,
        uptime: Duration::from_secs(3600 * 24 + 3600 * 2 + 30 * 60), // 1 day, 2 hours, 30 minutes
    };

    let alerts = vec![
        StatusAlert {
            level: AlertLevel::Warning,
            message: "Disk usage above 75%".to_string(),
            timestamp: Instant::now() - Duration::from_secs(300),
            source: "System Monitor".to_string(),
        },
        StatusAlert {
            level: AlertLevel::Info,
            message: "AI model cache updated".to_string(),
            timestamp: Instant::now() - Duration::from_secs(60),
            source: "AI Service".to_string(),
        },
    ];

    EnhancedPanel {
        id: "status".to_string(),
        title: "System Status".to_string(),
        content: PanelContent::Status { metrics, alerts },
        style_config: PanelStyleConfig::default(),
        scroll_position: 0,
        max_items: None,
        auto_scroll: false,
        visible: true,
        focused: false,
        last_updated: Instant::now(),
        error_count: 0,
        warning_count: 1,
    }
}

fn create_logs_panel() -> EnhancedPanel {
    let entries = vec![
        LogEntry {
            level: LogLevel::Info,
            message: "DevKit started successfully".to_string(),
            timestamp: Instant::now() - Duration::from_secs(3600),
            source: "main".to_string(),
            metadata: HashMap::new(),
        },
        LogEntry {
            level: LogLevel::Debug,
            message: "Loading configuration from ~/.devkit/config.toml".to_string(),
            timestamp: Instant::now() - Duration::from_secs(3599),
            source: "config".to_string(),
            metadata: HashMap::new(),
        },
        LogEntry {
            level: LogLevel::Warning,
            message: "API rate limit approaching (80% of limit used)".to_string(),
            timestamp: Instant::now() - Duration::from_secs(300),
            source: "ai_client".to_string(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("requests".to_string(), "400".to_string());
                meta.insert("limit".to_string(), "500".to_string());
                meta
            },
        },
        LogEntry {
            level: LogLevel::Error,
            message: "Failed to connect to remote AI service".to_string(),
            timestamp: Instant::now() - Duration::from_secs(120),
            source: "ai_client".to_string(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("endpoint".to_string(), "https://api.openai.com".to_string());
                meta.insert("status_code".to_string(), "503".to_string());
                meta
            },
        },
        LogEntry {
            level: LogLevel::Info,
            message: "Switched to local AI model".to_string(),
            timestamp: Instant::now() - Duration::from_secs(110),
            source: "ai_client".to_string(),
            metadata: HashMap::new(),
        },
    ];

    EnhancedPanel {
        id: "logs".to_string(),
        title: "Application Logs".to_string(),
        content: PanelContent::Logs {
            entries,
            level_filter: LogLevel::Debug,
            search_filter: None,
        },
        style_config: PanelStyleConfig::default(),
        scroll_position: 0,
        max_items: Some(500),
        auto_scroll: true,
        visible: true,
        focused: false,
        last_updated: Instant::now(),
        error_count: 1,
        warning_count: 1,
    }
}