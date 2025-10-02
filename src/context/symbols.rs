//! Symbol indexing and management for codebase context.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Type of symbol in the codebase
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SymbolType {
    Function,
    Method,
    Struct,
    Class,
    Interface,
    Enum,
    Variable,
    Constant,
    Module,
    Namespace,
    Type,
    Trait,
    Unknown,
}

impl std::fmt::Display for SymbolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolType::Function => write!(f, "Function"),
            SymbolType::Method => write!(f, "Method"),
            SymbolType::Struct => write!(f, "Struct"),
            SymbolType::Class => write!(f, "Class"),
            SymbolType::Interface => write!(f, "Interface"),
            SymbolType::Enum => write!(f, "Enum"),
            SymbolType::Variable => write!(f, "Variable"),
            SymbolType::Constant => write!(f, "Constant"),
            SymbolType::Module => write!(f, "Module"),
            SymbolType::Namespace => write!(f, "Namespace"),
            SymbolType::Type => write!(f, "Type"),
            SymbolType::Trait => write!(f, "Trait"),
            SymbolType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// A symbol definition in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub qualified_name: Option<String>,
    pub symbol_type: SymbolType,
    pub file_path: PathBuf,
    pub line: usize,
    pub line_number: usize, // Keep for backward compatibility
    pub column: usize,
    pub signature: Option<String>,
    pub documentation: Option<String>,
    pub visibility: Visibility,
    pub references: Vec<SymbolReference>,
}

/// Visibility of a symbol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Unknown,
}

/// A reference to a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolReference {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub column: usize,
    pub reference_type: ReferenceType,
}

/// Type of symbol reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReferenceType {
    Call,
    Definition,
    Import,
    Inheritance,
    Implementation,
    Usage,
    Unknown,
}

/// Index of all symbols in a codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolIndex {
    symbols: HashMap<String, Vec<Symbol>>,
    file_symbols: HashMap<PathBuf, Vec<String>>,
    symbol_count: usize,
}

impl SymbolIndex {
    /// Create a new empty symbol index
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            file_symbols: HashMap::new(),
            symbol_count: 0,
        }
    }

    /// Add a symbol to the index
    pub fn add_symbol(&mut self, symbol: Symbol) {
        let symbol_name = symbol.name.clone();
        let file_path = symbol.file_path.clone();

        // Add to main symbols map
        self.symbols
            .entry(symbol_name.clone())
            .or_insert_with(Vec::new)
            .push(symbol);

        // Add to file symbols map
        self.file_symbols
            .entry(file_path)
            .or_insert_with(Vec::new)
            .push(symbol_name);

        self.symbol_count += 1;
    }

    /// Find symbols by name
    pub fn find_symbols(&self, name: &str) -> Vec<&Symbol> {
        self.symbols
            .get(name)
            .map(|symbols| symbols.iter().collect())
            .unwrap_or_default()
    }

    /// Find symbols by type
    pub fn find_symbols_by_type(&self, symbol_type: &SymbolType) -> Vec<&Symbol> {
        self.symbols
            .values()
            .flatten()
            .filter(|symbol| symbol.symbol_type == *symbol_type)
            .collect()
    }

    /// Get symbols in a specific file
    pub fn get_file_symbols(&self, file_path: &PathBuf) -> Vec<&Symbol> {
        if let Some(symbol_names) = self.file_symbols.get(file_path) {
            symbol_names
                .iter()
                .filter_map(|name| self.symbols.get(name))
                .flatten()
                .filter(|symbol| symbol.file_path == *file_path)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Search symbols with fuzzy matching
    pub fn search(&self, query: &str, symbol_types: Option<&[SymbolType]>) -> Vec<Symbol> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for symbols in self.symbols.values() {
            for symbol in symbols {
                // Check if symbol type matches filter
                if let Some(types) = symbol_types {
                    if !types.contains(&symbol.symbol_type) {
                        continue;
                    }
                }

                // Simple fuzzy matching
                if symbol.name.to_lowercase().contains(&query_lower) {
                    results.push(symbol.clone());
                }
            }
        }

        // Sort by relevance (exact matches first, then partial matches)
        results.sort_by(|a, b| {
            let a_exact = a.name.to_lowercase() == query_lower;
            let b_exact = b.name.to_lowercase() == query_lower;

            match (a_exact, b_exact) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        results
    }

    /// Get all symbol names
    pub fn get_all_symbol_names(&self) -> Vec<String> {
        self.symbols.keys().cloned().collect()
    }

    /// Get total symbol count
    pub fn total_symbols(&self) -> usize {
        self.symbol_count
    }

    /// Get symbols by file
    pub fn get_files_with_symbols(&self) -> Vec<PathBuf> {
        self.file_symbols.keys().cloned().collect()
    }

    /// Remove symbols from a file (for updates)
    pub fn remove_file_symbols(&mut self, file_path: &PathBuf) {
        if let Some(symbol_names) = self.file_symbols.remove(file_path) {
            for symbol_name in symbol_names {
                if let Some(symbols) = self.symbols.get_mut(&symbol_name) {
                    symbols.retain(|symbol| symbol.file_path != *file_path);
                    if symbols.is_empty() {
                        self.symbols.remove(&symbol_name);
                    }
                }
            }
        }
    }

    /// Update symbols for a file
    pub fn update_file_symbols(&mut self, file_path: &PathBuf, new_symbols: Vec<Symbol>) {
        // Remove old symbols
        self.remove_file_symbols(file_path);

        // Add new symbols
        for symbol in new_symbols {
            self.add_symbol(symbol);
        }
    }
}

impl Symbol {
    /// Create a new symbol
    pub fn new(
        name: String,
        symbol_type: SymbolType,
        file_path: PathBuf,
        line_number: usize,
        column: usize,
    ) -> Self {
        Self {
            name,
            qualified_name: None,
            symbol_type,
            file_path,
            line: line_number,
            line_number,
            column,
            signature: None,
            documentation: None,
            visibility: Visibility::Unknown,
            references: Vec::new(),
        }
    }

    /// Add a reference to this symbol
    pub fn add_reference(&mut self, reference: SymbolReference) {
        self.references.push(reference);
    }

    /// Get all references of a specific type
    pub fn get_references_by_type(&self, ref_type: &ReferenceType) -> Vec<&SymbolReference> {
        self.references
            .iter()
            .filter(|reference| reference.reference_type == *ref_type)
            .collect()
    }

    /// Check if this symbol is public
    pub fn is_public(&self) -> bool {
        self.visibility == Visibility::Public
    }

    /// Get the qualified name (with file path context)
    pub fn qualified_name(&self) -> String {
        format!("{}:{}", self.file_path.to_string_lossy(), self.name)
    }
}

impl SymbolReference {
    /// Create a new symbol reference
    pub fn new(
        file_path: PathBuf,
        line_number: usize,
        column: usize,
        reference_type: ReferenceType,
    ) -> Self {
        Self {
            file_path,
            line_number,
            column,
            reference_type,
        }
    }
}

impl Default for SymbolIndex {
    fn default() -> Self {
        Self::new()
    }
}
