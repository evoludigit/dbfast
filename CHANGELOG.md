# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-09-13

### 🐛 Fixed
- **Critical**: Fixed SQL parsing error with PostgreSQL dollar-quoted strings (#29)
  - Properly handle DO $$ blocks and stored procedures
  - Support for custom dollar-quote tags (e.g., $BODY$, $func$)
  - Preserve statement integrity within dollar-quoted blocks
  - Comprehensive test coverage for edge cases and nested quotes
  - Resolves blocking issue for migrating 791 SQL files to DBFast

### ✨ Added
- **Environment Commands & Validation System** (Phase 3B)
  - New `dbfast environments` command to list configured environments
  - Enhanced `dbfast validate-env` command with production safety warnings
  - Verbose mode support for detailed environment information
  - File existence validation for SQL repositories
  - Environment filtering and directory exclusion support

- **Enhanced Documentation**
  - Comprehensive project documentation with examples
  - Improved README with detailed usage instructions
  - Better code documentation and API examples
  - Enhanced error messages and help text

### 🔧 Improved
- **Code Quality** (96% clippy warning reduction)
  - Eliminated enterprise theater and unnecessary complexity
  - Improved error handling and type safety
  - Better resource management and memory efficiency
  - Enhanced connection pooling and timeout handling

- **Database Operations**
  - Industrial-grade database name validation and security
  - Improved connection management with proper pool exhaustion prevention
  - Better transaction handling and rollback mechanisms
  - Enhanced SQL file scanning and change detection

- **Performance & Reliability**
  - Optimized file scanning with better change detection
  - Improved hash-based metadata storage and retrieval
  - Better error propagation and context information
  - Enhanced logging and debugging capabilities

### 🧪 Testing
- Added comprehensive test suites for SQL parsing edge cases
- Improved integration tests for environment commands
- Enhanced unit tests for database operations
- Better test coverage for error scenarios

### 📦 Dependencies
- Updated various dependencies for security and performance
- Improved dependency management and version constraints

---

## [0.1.0] - 2024-11-01

### ✨ Initial Release
- Core database connection pooling with PostgreSQL
- Basic SQL file scanning and execution
- Template-based database operations
- Configuration management with TOML
- CLI interface with essential commands
- Change detection for SQL files
- Basic backup and restoration features
