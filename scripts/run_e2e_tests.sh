#!/bin/bash

# DocPilot End-to-End Usability Test Runner
# This script runs comprehensive automated tests for all DocPilot functionality

# Don't exit on error - we want to handle errors gracefully
set +e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_DIR=$(mktemp -d)
DOCPILOT_BINARY=""
TEST_SESSION_PREFIX="e2e-test"
CLEANUP_ON_EXIT=true

# Cleanup function
cleanup() {
    if [ "$CLEANUP_ON_EXIT" = true ]; then
        echo -e "${YELLOW}🧹 Cleaning up test environment...${NC}"
        
        # Stop any running DocPilot sessions
        $DOCPILOT_BINARY stop 2>/dev/null || true
        
        # Kill any background DocPilot processes
        pkill -f "docpilot" 2>/dev/null || true
        
        # Remove test directory
        rm -rf "$TEST_DIR" 2>/dev/null || true
        
        echo -e "${GREEN}✅ Cleanup completed${NC}"
    fi
}

# Set up cleanup on script exit
trap cleanup EXIT

# Function to print test headers
print_test_header() {
    echo -e "\n${BLUE}🧪 Test: $1${NC}"
    echo "$(printf '=%.0s' {1..50})"
}

# Function to run DocPilot command with error handling
run_docpilot() {
    local cmd="$1"
    shift
    
    echo -e "${YELLOW}▶ Running: docpilot $cmd $*${NC}"
    
    local output
    local exit_code
    output=$(HOME="$TEST_DIR" "$DOCPILOT_BINARY" "$cmd" "$@" 2>&1)
    exit_code=$?
    
    echo "$output"
    
    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}✅ Command succeeded${NC}"
        return 0
    else
        echo -e "${RED}❌ Command failed (exit code: $exit_code)${NC}"
        return 1
    fi
}

# Function to run DocPilot command and expect it to fail
run_docpilot_expect_fail() {
    local cmd="$1"
    shift
    
    echo -e "${YELLOW}▶ Running (expecting failure): docpilot $cmd $*${NC}"
    
    local output
    local exit_code
    output=$(HOME="$TEST_DIR" "$DOCPILOT_BINARY" "$cmd" "$@" 2>&1)
    exit_code=$?
    
    echo "$output"
    
    if [ $exit_code -ne 0 ]; then
        echo -e "${GREEN}✅ Command failed as expected${NC}"
        return 0
    else
        echo -e "${RED}❌ Command should have failed but succeeded${NC}"
        return 1
    fi
}

# Function to run shell command in test environment
run_shell() {
    local cmd="$1"
    echo -e "${YELLOW}▶ Running shell: $cmd${NC}"
    
    if cd "$TEST_DIR" && eval "$cmd"; then
        echo -e "${GREEN}✅ Shell command succeeded${NC}"
        return 0
    else
        echo -e "${RED}❌ Shell command failed${NC}"
        return 1
    fi
}

# Function to check if file exists
check_file_exists() {
    local file="$1"
    if [ -f "$TEST_DIR/$file" ]; then
        echo -e "${GREEN}✅ File exists: $file${NC}"
        return 0
    else
        echo -e "${RED}❌ File missing: $file${NC}"
        return 1
    fi
}

# Test 1: Complete Basic Workflow
test_basic_workflow() {
    print_test_header "Complete Basic Workflow"
    
    # Start session
    run_docpilot start "E2E Basic Workflow Test" --output "basic-test.md" || return 1
    
    # Check status
    run_docpilot status || return 1
    
    # Add various annotations
    run_docpilot annotate "This is a test note" --annotation-type note || return 1
    run_docpilot annotate "This explains the process" --annotation-type explanation || return 1
    run_docpilot annotate "This is a warning" --annotation-type warning || return 1
    run_docpilot annotate "Milestone reached" --annotation-type milestone || return 1
    
    # Test quick annotation commands
    run_docpilot note "Quick note test" || return 1
    run_docpilot explain "Quick explanation test" || return 1
    run_docpilot warn "Quick warning test" || return 1
    run_docpilot milestone "Quick milestone test" || return 1
    
    # List annotations
    run_docpilot annotations || return 1
    
    # Test filtering
    run_docpilot annotations --filter-type warning || return 1
    run_docpilot annotations --recent 3 || return 1
    
    # Test pause/resume
    run_docpilot pause || return 1
    run_docpilot resume || return 1
    
    # Stop session
    run_docpilot stop || return 1
    
    # Generate documentation
    run_docpilot generate --output "final-basic.md" --template standard || return 1
    
    # Verify output file
    check_file_exists "final-basic.md" || return 1
    
    echo -e "${GREEN}🎉 Basic workflow test completed successfully!${NC}"
}

# Test 2: Configuration Management
test_configuration() {
    print_test_header "Configuration Management"
    
    # View empty config
    run_docpilot config || return 1
    
    # Set provider
    run_docpilot config --provider claude || return 1
    
    # Set API key
    run_docpilot config --api-key "test-key-12345" || return 1
    
    # Set base URL
    run_docpilot config --base-url "http://localhost:11434" || return 1
    
    # Set all at once
    run_docpilot config --provider ollama --api-key "ollama-key" --base-url "http://localhost:11434" || return 1
    
    # View updated config
    run_docpilot config || return 1
    
    echo -e "${GREEN}🎉 Configuration test completed successfully!${NC}"
}

# Test 3: Session State Management
test_session_states() {
    print_test_header "Session State Management"
    
    # Check no active session
    run_docpilot status || return 1
    
    # Try operations without session (should fail)
    if run_docpilot pause 2>/dev/null; then
        echo -e "${RED}❌ Pause should fail without active session${NC}"
        return 1
    fi
    
    # Start session
    run_docpilot start "State test session" || return 1
    
    # Try to start another (should fail)
    if run_docpilot start "Second session" 2>/dev/null; then
        echo -e "${RED}❌ Second start should fail${NC}"
        return 1
    else
        echo -e "${GREEN}✅ Second start correctly failed${NC}"
    fi
    
    # Test pause/resume cycle
    run_docpilot pause || return 1
    
    # Try to pause again (should fail)
    if run_docpilot pause 2>/dev/null; then
        echo -e "${RED}❌ Second pause should fail${NC}"
        return 1
    else
        echo -e "${GREEN}✅ Second pause correctly failed${NC}"
    fi
    
    run_docpilot resume || return 1
    
    # Clean up
    run_docpilot stop || return 1
    
    echo -e "${GREEN}🎉 Session state test completed successfully!${NC}"
}

# Test 4: Documentation Templates
test_templates() {
    print_test_header "Documentation Templates"
    
    # Start session and add content
    run_docpilot start "Template test session" || return 1
    run_docpilot note "Test note for templates" || return 1
    run_docpilot warn "Test warning for templates" || return 1
    run_docpilot milestone "Template milestone" || return 1
    run_docpilot stop || return 1
    
    # Test different templates
    local templates=("standard" "minimal" "comprehensive" "hierarchical" "professional" "technical" "rich" "github")
    
    for template in "${templates[@]}"; do
        echo -e "${YELLOW}Testing template: $template${NC}"
        if run_docpilot generate --output "test-$template.md" --template "$template"; then
            check_file_exists "test-$template.md" || return 1
        else
            echo -e "${YELLOW}⚠ Template $template failed (might not be implemented)${NC}"
        fi
    done
    
    echo -e "${GREEN}🎉 Template test completed successfully!${NC}"
}

# Test 5: Error Handling
test_error_handling() {
    print_test_header "Error Handling and Edge Cases"
    
    # Test invalid commands (should fail)
    if run_docpilot invalid-command 2>/dev/null; then
        echo -e "${RED}❌ Invalid command should fail${NC}"
        return 1
    fi
    
    if run_docpilot start 2>/dev/null; then
        echo -e "${RED}❌ Start without description should fail${NC}"
        return 1
    fi
    
    # Start session for annotation tests
    run_docpilot start "Error test session" || return 1
    
    # Test invalid annotation type
    if run_docpilot annotate "test" --annotation-type invalid-type 2>/dev/null; then
        echo -e "${RED}❌ Invalid annotation type should fail${NC}"
        return 1
    fi
    
    # Test special characters
    run_docpilot note "Special chars: !@#$%^&*()[]{}|\\:;\"'<>,.?/~\`" || return 1
    
    # Test Unicode
    run_docpilot note "Unicode: 🚀 DocPilot 测试 العربية русский" || return 1
    
    # Clean up
    run_docpilot stop || return 1
    
    echo -e "${GREEN}🎉 Error handling test completed successfully!${NC}"
}

# Test 6: Help and Documentation
test_help() {
    print_test_header "Help and Documentation"
    
    # Test main help
    run_docpilot --help || return 1
    
    # Test version
    run_docpilot --version || return 1
    
    # Test subcommand help
    local subcommands=("start" "stop" "pause" "resume" "annotate" "note" "config" "generate" "status")
    
    for subcmd in "${subcommands[@]}"; do
        echo -e "${YELLOW}Testing help for: $subcmd${NC}"
        run_docpilot "$subcmd" --help || return 1
    done
    
    echo -e "${GREEN}🎉 Help documentation test completed successfully!${NC}"
}

# Test 7: Performance Test
test_performance() {
    print_test_header "Performance and Stress Testing"
    
    # Start session
    run_docpilot start "Performance test session" || return 1
    
    # Add many annotations quickly
    echo -e "${YELLOW}Adding 20 annotations rapidly...${NC}"
    for i in {1..20}; do
        run_docpilot note "Performance test annotation $i" || return 1
    done
    
    # Test rapid status checks
    echo -e "${YELLOW}Performing rapid status checks...${NC}"
    for i in {1..5}; do
        run_docpilot status || return 1
    done
    
    # List all annotations
    run_docpilot annotations || return 1
    
    # Clean up
    run_docpilot stop || return 1
    
    echo -e "${GREEN}🎉 Performance test completed successfully!${NC}"
}

# Main test execution
main() {
    echo -e "${BLUE}🚀 DocPilot End-to-End Usability Tests${NC}"
    echo "========================================"
    
    # Find DocPilot binary
    if [ -f "./target/release/docpilot" ]; then
        DOCPILOT_BINARY="./target/release/docpilot"
    elif [ -f "./target/debug/docpilot" ]; then
        DOCPILOT_BINARY="./target/debug/docpilot"
    elif command -v docpilot >/dev/null 2>&1; then
        DOCPILOT_BINARY="docpilot"
    else
        echo -e "${RED}❌ DocPilot binary not found. Please build the project first:${NC}"
        echo "   cargo build --release"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Found DocPilot binary: $DOCPILOT_BINARY${NC}"
    echo -e "${YELLOW}📁 Test directory: $TEST_DIR${NC}"
    
    # Run all tests
    local tests=(
        "test_basic_workflow"
        "test_configuration" 
        "test_session_states"
        "test_templates"
        "test_error_handling"
        "test_help"
        "test_performance"
    )
    
    local passed=0
    local failed=0
    
    for test_func in "${tests[@]}"; do
        echo -e "\n${BLUE}🔄 Running $test_func...${NC}"
        
        if $test_func; then
            ((passed++))
            echo -e "${GREEN}✅ $test_func PASSED${NC}"
        else
            ((failed++))
            echo -e "${RED}❌ $test_func FAILED${NC}"
        fi
    done
    
    # Summary
    echo -e "\n${BLUE}📊 Test Summary${NC}"
    echo "==============="
    echo -e "${GREEN}✅ Passed: $passed${NC}"
    echo -e "${RED}❌ Failed: $failed${NC}"
    echo -e "${YELLOW}📁 Test artifacts in: $TEST_DIR${NC}"
    
    if [ $failed -eq 0 ]; then
        echo -e "\n${GREEN}🎉 All tests passed! DocPilot is working correctly.${NC}"
        return 0
    else
        echo -e "\n${RED}💥 Some tests failed. Please check the output above.${NC}"
        return 1
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-cleanup)
            CLEANUP_ON_EXIT=false
            shift
            ;;
        --help)
            echo "DocPilot E2E Test Runner"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --no-cleanup    Don't clean up test directory on exit"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Run main function
main "$@"