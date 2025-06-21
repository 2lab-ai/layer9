# LEARNING
- When user points out mistakes, automatically update local CLAUDE.LOCAL.md with the lesson learned

# STATUS REPORT COMMAND
When user says "현재 프로젝트 리포트" or "project status report" or 비슷한 의미.
1. Create a comprehensive report in ./reports/ directory with timestamp
2. Include:
   - 전체 프로젝트 할일에서 한일과 남은일 (Done vs remaining tasks)
   - 남은 마일스톤 정리 (Remaining milestones)  
   - 당장 해야할일 (Immediate tasks)
   - 네가 생각하기에 해야할일 (What I think should be done)
3. Use ultrathink for comprehensive analysis

# TESTING
- Always test your code before showing results to the user
- Run make --dry-run for Makefiles, cargo check for Rust, syntax validation for configs

# BUILD
- SQLx compile-time verification requires database - use SQLX_OFFLINE=true or --no-default-features to skip
- Workspace Cargo.toml inheritance is complex - skip it and build from subdirectory instead
- Convert all sqlx::query! to sqlx::query with manual binding to avoid compile-time DB checks

# OPERATION ABSTRACTION
- Code abstraction ≠ Operation abstraction
- Multiple tools (Docker, Make, Cargo, psql) = High cognitive load
- Need single control plane for entire system
- Self-healing > Manual debugging
- Best abstraction is the one you don't notice
- Implemented: mira command unifies all operations (doctor, up, down, status, logs, shell, reset)

# TESTING AND VALIDATION
- ALWAYS test health check endpoints before implementing them
- Don't assume service health endpoints - verify actual API
- Qdrant uses / root endpoint for health check, NOT /health or /readiness
- Timeout handling must be configurable, not hardcoded
- If service is working but health check fails, the health check is wrong, not the service
- ALWAYS verify changes are actually applied - don't assume config changes take effect
- Clear error messages are critical - show WHICH services are unhealthy
- Test the entire flow end-to-end, not just individual components

# PROCESS MANAGEMENT
- Native processes need proper lifecycle management - spawn, track PID, monitor, cleanup
- Environment variables must be explicitly loaded from .env files when spawning processes
- Use PID files to track running processes across sessions
- Background processes need proper stdout/stderr handling to avoid zombies
- Consider using systemd/supervisor for production, but simple PID tracking works for POC
- CRITICAL: When spawning Rust processes, redirect stdout/stderr to files to prevent blocking
- Use tokio::spawn to detach child processes properly for background execution
- Always verify process is actually running with ps after starting - don't trust spawn alone