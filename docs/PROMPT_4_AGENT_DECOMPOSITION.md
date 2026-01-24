# 4-Agent Decomposition Prompt

This prompt breaks down a Daily Ambition into exactly FOUR independent, non-overlapping workstreams that can run in parallel.

**Source:** `backend/services/agent_prompt_decomposer.py` - `_analyze_and_assign_work()` method

**Purpose:** Generate 4 domain-specific agent prompts with zero file overlap, ensuring parallel execution.

---

## The Prompt

```
You are analyzing a Daily Ambition document to break it into exactly FOUR independent workstreams that can run in parallel.

## Daily Ambition

{daily_ambition_text[:3000]}

{*[Truncated - analyzing first 3000 chars]* if daily_ambition_text > 3000 chars}

## Portfolio Context

{portfolio_context if provided else "No portfolio context provided."}

## Yesterday's Context

{yesterday_context if provided else "No yesterday context provided."}

## Your Task

Break down the daily ambition into EXACTLY FOUR independent workstreams:

1. **Frontend Development**: UI components, client-side logic, mobile responsive, React/JSX files
2. **Backend Development**: Server logic, APIs, data processing, Python/FastAPI files  
3. **Security & Quality**: Code reviews, testing, compliance, validation, linting
4. **System Maintenance**: PAM monitoring, health checks, infrastructure improvements

## Critical Rules

1. **NO FILE OVERLAP**: Each domain must work on DIFFERENT files
   - Frontend: frontend/**/*.jsx, frontend/**/*.js, frontend/**/*.css
   - Backend: backend/**/*.py, backend/routers/**, backend/services/**
   - Security & Quality: **/*test*.py, **/*test*.js, linting configs
   - System Maintenance: Dockerfiles, cloudbuild.yaml, deploy scripts, monitoring

2. **INDEPENDENCE**: Each workstream must be completable without waiting on others
   - If work naturally spans domains, split at logical boundaries
   - Document integration points but don't create blocking dependencies

3. **COMPLETE ASSIGNMENTS**: Each domain should have:
   - Clear objective (1-2 sentences)
   - Specific files to modify (list file paths)
   - Success criteria (3-5 measurable items)
   - Dependencies (if any - should be minimal)
   - Integration points (how this connects to other work)

4. **REALISTIC SCOPE**: Each workstream should be ~8 hours of work (37-72 microtasks)

## Output Format

Return ONLY valid JSON (no markdown, no explanation):

```json
{
  "frontend": {
    "objective": "Clear objective for frontend work",
    "files_to_modify": ["frontend/src/pages/Dashboard.jsx", "frontend/src/components/..."],
    "success_criteria": ["Criterion 1", "Criterion 2", "..."],
    "dependencies": ["Dependency 1 if any", "..."],
    "integration_points": ["How this connects to backend", "..."]
  },
  "backend": {
    "objective": "Clear objective for backend work",
    "files_to_modify": ["backend/routers/...", "backend/services/..."],
    "success_criteria": ["Criterion 1", "Criterion 2", "..."],
    "dependencies": [],
    "integration_points": ["How this connects to frontend", "..."]
  },
  "security_quality": {
    "objective": "Clear objective for security/quality work",
    "files_to_modify": ["**/*test*.py", "..."],
    "success_criteria": ["Criterion 1", "Criterion 2", "..."],
    "dependencies": [],
    "integration_points": []
  },
  "system_maintenance": {
    "objective": "Clear objective for system maintenance work",
    "files_to_modify": ["backend/Dockerfile", "..."],
    "success_criteria": ["Criterion 1", "Criterion 2", "..."],
    "dependencies": [],
    "integration_points": []
  }
}
```

**CRITICAL**: Ensure NO file path appears in multiple domains. If daily ambition has <4 workstreams, assign remaining domains minimal placeholder work or combine related work intelligently.

Return ONLY the JSON, no other text:
```

---

## Key Features

1. **Zero File Overlap**: Enforces strict file separation across domains
2. **Parallel Execution**: Each workstream is independent and can run simultaneously
3. **Complete Assignments**: Each domain gets objectives, files, success criteria, dependencies, and integration points
4. **Realistic Scope**: ~8 hours per workstream (37-72 microtasks)
5. **Structured Output**: Returns clean JSON for easy parsing and prompt generation

## Usage

This prompt is called automatically when generating agent prompts from a Daily Ambition via:
- Frontend: `/goals` page → "Generate Agent Prompts" button
- Backend: `POST /api/agent-prompts/generate` endpoint
- Service: `AgentPromptDecomposer.decompose_daily_ambition()`

## Validation

After generation, the system validates:
- No file overlap across domains (`_validate_no_file_overlap()`)
- All 4 domains have assignments
- JSON structure is valid

## Example Output

```json
{
  "frontend": {
    "objective": "Build responsive dashboard components for meeting proposals",
    "files_to_modify": [
      "frontend/src/pages/MeetingProposals.jsx",
      "frontend/src/components/MeetingProposalList.jsx",
      "frontend/src/components/MeetingProposalItem.jsx"
    ],
    "success_criteria": [
      "All components render without errors",
      "Mobile responsive design works on <768px screens",
      "API integration complete with error handling"
    ],
    "dependencies": [],
    "integration_points": ["Uses backend API endpoint /api/meeting-proposals"]
  },
  "backend": {
    "objective": "Create meeting proposals API endpoints and service layer",
    "files_to_modify": [
      "backend/routers/meeting_proposals.py",
      "backend/services/meeting_proposal_service.py"
    ],
    "success_criteria": [
      "GET /api/meeting-proposals returns list",
      "POST /api/meeting-proposals creates new proposal",
      "Database schema updated with proposals table"
    ],
    "dependencies": [],
    "integration_points": ["Frontend consumes these endpoints"]
  },
  "security_quality": {
    "objective": "Add unit tests and security validation for meeting proposals",
    "files_to_modify": [
      "tests/test_meeting_proposals.py",
      "backend/services/meeting_proposal_service.py"
    ],
    "success_criteria": [
      "Test coverage >80%",
      "Input validation prevents SQL injection",
      "Authentication required for all endpoints"
    ],
    "dependencies": ["Backend endpoints must exist"],
    "integration_points": []
  },
  "system_maintenance": {
    "objective": "Update monitoring and health checks for new endpoints",
    "files_to_modify": [
      "backend/monitoring/health_checks.py",
      "cloudbuild.yaml"
    ],
    "success_criteria": [
      "Health check includes meeting-proposals endpoint",
      "Deployment pipeline updated",
      "Logging configured for new endpoints"
    ],
    "dependencies": [],
    "integration_points": []
  }
}
```

---

**Last Updated:** 2025-01-21  
**File:** `backend/services/agent_prompt_decomposer.py:171-256`  
**LLM Service:** Gemini (via `GeminiService`)  
**Rate Limit:** 4 seconds between calls

