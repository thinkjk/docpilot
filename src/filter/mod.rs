//! Command filtering and validation module
//! 
//! This module provides functionality to filter commands based on various criteria
//! including success/failure detection, command validation, and privacy filtering.

pub mod command;

pub use command::{
    CommandFilter, FilterCriteria, FilterResult, FilteringStats,
    WorkflowOptimization, OptimizationType, ProcessedCommands, PrivacyMode,
    CommandDependency, ValidationResult, ValidationType, SequenceValidationError, ValidationErrorType
};