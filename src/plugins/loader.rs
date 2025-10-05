//! Plugin Loader
//!
//! Handles dynamic loading and unloading of plugins from various sources including
//! native libraries, WebAssembly modules, and scripted plugins.

use crate::plugins::{
    Plugin, PluginError, PluginMetadata, PluginSystemConfig, PluginCapability, PluginHealth
};
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::{CString, OsStr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::process::Command;
use tracing::{debug, error, info, warn};

/// Plugin loader factory that creates appropriate loaders for different plugin types
pub struct PluginLoaderFactory;

impl PluginLoaderFactory {
    /// Create a loader based on plugin metadata and system config
    pub fn create_loader(
        plugin_path: &Path,
        metadata: &PluginMetadata,
        config: &PluginSystemConfig,
    ) -> Result<Box<dyn PluginLoader + Send + Sync>, PluginError> {
        // Determine plugin type from manifest or file extension
        let plugin_type = Self::detect_plugin_type(plugin_path, metadata)?;
        
        match plugin_type {
            PluginType::Native => {
                Ok(Box::new(NativePluginLoader::new(config.clone())))
            }
            PluginType::Wasm => {
                Ok(Box::new(WasmPluginLoader::new(config.clone())))
            }
            PluginType::Script => {
                Ok(Box::new(ScriptPluginLoader::new(config.clone())))
            }
            PluginType::Python => {
                Ok(Box::new(PythonPluginLoader::new(config.clone())))
            }
        }
    }
    
    fn detect_plugin_type(plugin_path: &Path, metadata: &PluginMetadata) -> Result<PluginType, PluginError> {
        // Check entry point extension
        let entry_path = plugin_path.join(&metadata.entry_point);
        
        if let Some(extension) = entry_path.extension().and_then(OsStr::to_str) {
            match extension.to_lowercase().as_str() {
                "so" | "dylib" | "dll" => Ok(PluginType::Native),
                "wasm" => Ok(PluginType::Wasm),
                "py" => Ok(PluginType::Python),
                "js" | "ts" => Ok(PluginType::Script),
                _ => Err(PluginError::LoadFailed(
                    format!("Unsupported plugin type: {}", extension)
                )),
            }
        } else {
            Err(PluginError::LoadFailed("Could not detect plugin type".to_string()))
        }
    }
}

/// Plugin type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum PluginType {
    Native,
    Wasm,
    Script,
    Python,
}

/// Plugin loader trait for different plugin types
#[async_trait::async_trait]
pub trait PluginLoader {
    /// Load a plugin from the given path
    async fn load_plugin(&self, path: &Path, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>, PluginError>;
    
    /// Unload a plugin
    async fn unload_plugin(&self, plugin_id: &str) -> Result<(), PluginError>;
    
    /// Check if a plugin can be loaded
    fn can_load(&self, path: &Path) -> bool;
    
    /// Get supported plugin extensions
    fn supported_extensions(&self) -> Vec<&'static str>;
    
    /// Hot reload a plugin
    async fn hot_reload(&self, plugin_id: &str, path: &Path, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>, PluginError> {
        // Default implementation: unload and reload
        self.unload_plugin(plugin_id).await?;
        self.load_plugin(path, metadata).await
    }
}

/// Native plugin loader for shared libraries (.so, .dylib, .dll)
pub struct NativePluginLoader {
    config: PluginSystemConfig,
    loaded_libraries: Arc<RwLock<HashMap<String, Library>>>,
}

impl NativePluginLoader {
    pub fn new(config: PluginSystemConfig) -> Self {
        Self {
            config,
            loaded_libraries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Load a native plugin using libloading
    async fn load_native_library(&self, path: &Path, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>, PluginError> {
        let library_path = path.join(&metadata.entry_point);
        
        debug!("Loading native library: {:?}", library_path);
        
        // Load the library
        let library = unsafe {
            Library::new(&library_path).map_err(|e| {
                PluginError::LoadFailed(format!("Failed to load library: {}", e))
            })?
        };
        
        // Get the plugin creation function
        let create_plugin: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> = unsafe {
            library.get(b"create_plugin").map_err(|e| {
                PluginError::LoadFailed(format!("Failed to find create_plugin function: {}", e))
            })?
        };
        
        // Create the plugin instance
        let plugin_ptr = unsafe { create_plugin() };
        
        if plugin_ptr.is_null() {
            return Err(PluginError::LoadFailed("Plugin creation returned null".to_string()));
        }
        
        // For now, create a proxy plugin since we can't easily manage raw pointers
        let proxy_plugin = NativePluginProxy::new(metadata.clone(), library_path);
        
        // Store the library to keep it loaded
        {
            let mut libraries = self.loaded_libraries.write().unwrap();
            libraries.insert(metadata.id.clone(), library);
        }
        
        Ok(Box::new(proxy_plugin))
    }
}

#[async_trait::async_trait]
impl PluginLoader for NativePluginLoader {
    async fn load_plugin(&self, path: &Path, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>, PluginError> {
        self.load_native_library(path, metadata).await
    }

    async fn unload_plugin(&self, plugin_id: &str) -> Result<(), PluginError> {
        let mut libraries = self.loaded_libraries.write().unwrap();
        libraries.remove(plugin_id);
        debug!("Unloaded native plugin: {}", plugin_id);
        Ok(())
    }

    fn can_load(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.supported_extensions().contains(&ext))
            .unwrap_or(false)
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        if cfg!(target_os = "linux") {
            vec!["so"]
        } else if cfg!(target_os = "macos") {
            vec!["dylib"]
        } else if cfg!(target_os = "windows") {
            vec!["dll"]
        } else {
            vec![]
        }
    }
}

/// WebAssembly plugin loader
pub struct WasmPluginLoader {
    config: PluginSystemConfig,
    #[allow(dead_code)]
    loaded_modules: Arc<RwLock<HashMap<String, Vec<u8>>>>, // Store WASM bytecode for now
}

impl WasmPluginLoader {
    pub fn new(config: PluginSystemConfig) -> Self {
        Self {
            config,
            loaded_modules: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl PluginLoader for WasmPluginLoader {
    async fn load_plugin(&self, path: &Path, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>, PluginError> {
        // For now, create a mock WASM plugin
        // TODO: Implement actual WASM loading with wasmtime
        info!("Loading WASM plugin: {}", metadata.id);
        Ok(Box::new(WasmPluginProxy::new(metadata.clone())))
    }

    async fn unload_plugin(&self, plugin_id: &str) -> Result<(), PluginError> {
        debug!("Unloading WASM plugin: {}", plugin_id);
        Ok(())
    }

    fn can_load(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase() == "wasm")
            .unwrap_or(false)
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["wasm"]
    }
}

/// Script plugin loader for JavaScript/TypeScript
pub struct ScriptPluginLoader {
    config: PluginSystemConfig,
}

impl ScriptPluginLoader {
    pub fn new(config: PluginSystemConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl PluginLoader for ScriptPluginLoader {
    async fn load_plugin(&self, path: &Path, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>, PluginError> {
        info!("Loading script plugin: {}", metadata.id);
        Ok(Box::new(ScriptPluginProxy::new(metadata.clone(), path.to_owned())))
    }

    async fn unload_plugin(&self, plugin_id: &str) -> Result<(), PluginError> {
        debug!("Unloading script plugin: {}", plugin_id);
        Ok(())
    }

    fn can_load(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext.to_lowercase().as_str(), "js" | "ts"))
            .unwrap_or(false)
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["js", "ts"]
    }
}

/// Python plugin loader
pub struct PythonPluginLoader {
    config: PluginSystemConfig,
}

impl PythonPluginLoader {
    pub fn new(config: PluginSystemConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl PluginLoader for PythonPluginLoader {
    async fn load_plugin(&self, path: &Path, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>, PluginError> {
        info!("Loading Python plugin: {}", metadata.id);
        Ok(Box::new(PythonPluginProxy::new(metadata.clone(), path.to_owned())))
    }

    async fn unload_plugin(&self, plugin_id: &str) -> Result<(), PluginError> {
        debug!("Unloading Python plugin: {}", plugin_id);
        Ok(())
    }

    fn can_load(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase() == "py")
            .unwrap_or(false)
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["py"]
    }
}

/// Native plugin proxy that manages the lifecycle of a native plugin
pub struct NativePluginProxy {
    metadata: PluginMetadata,
    library_path: PathBuf,
}

impl NativePluginProxy {
    pub fn new(metadata: PluginMetadata, library_path: PathBuf) -> Self {
        Self { metadata, library_path }
    }
}

#[async_trait::async_trait]
impl Plugin for NativePluginProxy {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn version(&self) -> &str {
        &self.metadata.version
    }

    async fn initialize(&mut self) -> Result<(), PluginError> {
        debug!("Initializing native plugin: {}", self.metadata.id);
        // TODO: Call native plugin initialize function
        Ok(())
    }

    async fn execute(&mut self, input: &str) -> Result<String, PluginError> {
        debug!("Executing native plugin: {} with input: {}", self.metadata.id, input);
        // TODO: Call native plugin execute function
        Ok(format!("Native plugin {} processed: {}", self.metadata.id, input))
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        debug!("Shutting down native plugin: {}", self.metadata.id);
        // TODO: Call native plugin shutdown function
        Ok(())
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        self.metadata.capabilities.clone()
    }
}

impl std::fmt::Debug for NativePluginProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NativePluginProxy {{ id: {} }}", self.metadata.id)
    }
}

/// WASM plugin proxy
pub struct WasmPluginProxy {
    metadata: PluginMetadata,
}

impl WasmPluginProxy {
    pub fn new(metadata: PluginMetadata) -> Self {
        Self { metadata }
    }
}

#[async_trait::async_trait]
impl Plugin for WasmPluginProxy {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn version(&self) -> &str {
        &self.metadata.version
    }

    async fn initialize(&mut self) -> Result<(), PluginError> {
        debug!("Initializing WASM plugin: {}", self.metadata.id);
        Ok(())
    }

    async fn execute(&mut self, input: &str) -> Result<String, PluginError> {
        debug!("Executing WASM plugin: {} with input: {}", self.metadata.id, input);
        Ok(format!("WASM plugin {} processed: {}", self.metadata.id, input))
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        debug!("Shutting down WASM plugin: {}", self.metadata.id);
        Ok(())
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        self.metadata.capabilities.clone()
    }
}

impl std::fmt::Debug for WasmPluginProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WasmPluginProxy {{ id: {} }}", self.metadata.id)
    }
}

/// Script plugin proxy for JavaScript/TypeScript plugins
pub struct ScriptPluginProxy {
    metadata: PluginMetadata,
    script_path: PathBuf,
}

impl ScriptPluginProxy {
    pub fn new(metadata: PluginMetadata, script_path: PathBuf) -> Self {
        Self { metadata, script_path }
    }

    async fn execute_script(&self, method: &str, args: &[&str]) -> Result<String, PluginError> {
        let entry_script = self.script_path.join(&self.metadata.entry_point);
        
        // Execute with Node.js
        let output = Command::new("node")
            .arg(&entry_script)
            .arg(method)
            .args(args)
            .output()
            .await
            .map_err(|e| PluginError::ExecutionFailed(format!("Failed to execute script: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(PluginError::ExecutionFailed(format!("Script execution failed: {}", error)));
        }

        let result = String::from_utf8_lossy(&output.stdout);
        Ok(result.trim().to_string())
    }
}

#[async_trait::async_trait]
impl Plugin for ScriptPluginProxy {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn version(&self) -> &str {
        &self.metadata.version
    }

    async fn initialize(&mut self) -> Result<(), PluginError> {
        debug!("Initializing script plugin: {}", self.metadata.id);
        self.execute_script("initialize", &[]).await?;
        Ok(())
    }

    async fn execute(&mut self, input: &str) -> Result<String, PluginError> {
        debug!("Executing script plugin: {} with input: {}", self.metadata.id, input);
        self.execute_script("execute", &[input]).await
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        debug!("Shutting down script plugin: {}", self.metadata.id);
        self.execute_script("shutdown", &[]).await?;
        Ok(())
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        self.metadata.capabilities.clone()
    }
}

impl std::fmt::Debug for ScriptPluginProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScriptPluginProxy {{ id: {} }}", self.metadata.id)
    }
}

/// Python plugin proxy
pub struct PythonPluginProxy {
    metadata: PluginMetadata,
    script_path: PathBuf,
}

impl PythonPluginProxy {
    pub fn new(metadata: PluginMetadata, script_path: PathBuf) -> Self {
        Self { metadata, script_path }
    }

    async fn execute_python(&self, method: &str, args: &[&str]) -> Result<String, PluginError> {
        let entry_script = self.script_path.join(&self.metadata.entry_point);
        
        // Execute with Python
        let output = Command::new("python3")
            .arg(&entry_script)
            .arg(method)
            .args(args)
            .output()
            .await
            .map_err(|e| PluginError::ExecutionFailed(format!("Failed to execute Python script: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(PluginError::ExecutionFailed(format!("Python execution failed: {}", error)));
        }

        let result = String::from_utf8_lossy(&output.stdout);
        Ok(result.trim().to_string())
    }
}

#[async_trait::async_trait]
impl Plugin for PythonPluginProxy {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn version(&self) -> &str {
        &self.metadata.version
    }

    async fn initialize(&mut self) -> Result<(), PluginError> {
        debug!("Initializing Python plugin: {}", self.metadata.id);
        self.execute_python("initialize", &[]).await?;
        Ok(())
    }

    async fn execute(&mut self, input: &str) -> Result<String, PluginError> {
        debug!("Executing Python plugin: {} with input: {}", self.metadata.id, input);
        self.execute_python("execute", &[input]).await
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        debug!("Shutting down Python plugin: {}", self.metadata.id);
        self.execute_python("shutdown", &[]).await?;
        Ok(())
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        self.metadata.capabilities.clone()
    }
}

impl std::fmt::Debug for PythonPluginProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PythonPluginProxy {{ id: {} }}", self.metadata.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_plugin_loader_factory() {
        let temp_dir = TempDir::new().unwrap();
        let config = PluginSystemConfig::default();
        
        // Create a mock metadata for a native plugin
        let metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            tags: vec![],
            dependencies: vec![],
            permissions: vec![],
            entry_point: "plugin.so".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            capabilities: vec![PluginCapability::CodeAnalysis],
        };

        let loader = PluginLoaderFactory::create_loader(
            temp_dir.path(),
            &metadata,
            &config,
        );

        assert!(loader.is_ok());
        let loader = loader.unwrap();
        assert!(loader.supported_extensions().contains(&"so") || 
                loader.supported_extensions().contains(&"dylib") ||
                loader.supported_extensions().contains(&"dll"));
    }
}