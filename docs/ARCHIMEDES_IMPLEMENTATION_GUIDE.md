# Archimedes Voice Daily Ambition System - Implementation Guide
*Created: 2025-01-23*

## Overview

Archimedes is a voice-based AI assistant that helps prepare Daily Ambition statements through conversational interviews. The system uses a two-agent architecture:

1. **Voice Archimedes** - Real-time conversational AI using Ultravox for voice interaction
2. **Silent Archimedes** - Background processor that transforms transcripts into structured documents

This guide documents the complete implementation for porting to a Rust-based operating system.

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        USER INTERACTION                              │
└─────────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    VOICE ARCHIMEDES (Ultravox)                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                 │
│  │   WebRTC    │  │   System    │  │  Transcript │                 │
│  │   Session   │──│   Prompt    │──│   Capture   │                 │
│  └─────────────┘  └─────────────┘  └─────────────┘                 │
│       Voice: Cassidy-English | Max Duration: 60min                  │
└─────────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    WORKING MEMORY CONTEXT                            │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Last 5 Days Daily Ambitions (Google Drive file_ids)         │   │
│  │  Last 5 Days Transcripts (voice_prep_transcripts table)      │   │
│  │  Today's GitHub Commits (git log integration)                 │   │
│  │  Book Reports & Documentation (filtered by date)              │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   SILENT ARCHIMEDES (Background)                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                 │
│  │  Transcript │  │  Document   │  │   Vector    │                 │
│  │  Processor  │──│  Generator  │──│   Storage   │                 │
│  └─────────────┘  └─────────────┘  └─────────────┘                 │
│       Gemini LLM | Structured Output | ChromaDB                     │
└─────────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    4 AGENT PROMPT DECOMPOSER                         │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐       │
│  │  Frontend  │ │  Backend   │ │  Security  │ │   System   │       │
│  │   Agent    │ │   Agent    │ │   Agent    │ │   Agent    │       │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘       │
│       NO FILE OVERLAP ENFORCED | Independent Workstreams             │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Section 1: Voice Archimedes - The Conversational Agent

### 1.1 Ultravox Integration

**File**: `backend/services/ultravox_service.py`

```python
# Core Configuration
ULTRAVOX_API_BASE = 'https://api.ultravox.ai/api'

def create_webrtc_call(
    system_prompt: str,
    voice: str = "Cassidy-English",    # Default voice
    tools: Optional[List[Dict]] = None,
    max_duration_minutes: int = 60     # 1 hour max
) -> Dict:
    """
    Create WebRTC voice session via Ultravox API

    Returns:
    - callId: Ultravox call identifier
    - joinUrl: WebRTC URL for frontend connection
    """
    payload = {
        "systemPrompt": system_prompt,
        "model": "fixie-ai/ultravox",
        "voice": voice,
        "maxDuration": f"{max_duration_minutes * 60}s",
        "medium": {"webRtc": {}}
    }

    # POST to /api/calls
    response = requests.post(
        f"{ULTRAVOX_API_BASE}/calls",
        headers={"X-API-Key": api_key},
        json=payload
    )
    return response.json()
```

### 1.2 Critical API Calls

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `POST /api/calls` | Ultravox | Create voice session |
| `GET /api/daily-ambition/end-of-day/voice-session` | Backend | Initialize EOD session with context |
| `GET /api/daily-ambition/end-of-week/voice-session` | Backend | Initialize weekly summary session |

### 1.3 Session Initialization Flow

```python
# From backend/routers/daily_ambition.py:34-183

# 1. Generate unique session ID
session_id = str(uuid.uuid4())

# 2. Load context from services
from backend.services.end_of_day_context_service import build_end_of_day_context
from backend.services.end_of_day_prompt_service import build_end_of_day_prompt

context = build_end_of_day_context(user_id, user_email, days=5)
prompt = build_end_of_day_prompt(context)

# 3. Create Ultravox session with system prompt
ultravox_session = ultravox.create_session(
    session_id=session_id,
    user_id=user_id,
    context={'event_id': event_id, 'prep_type': 'end_of_day'},
    system_prompt=prompt
)

# 4. Create transcript record in database
INSERT INTO voice_prep_transcripts
(session_id, event_id, user_id, prep_type, transcript_text, transcript_json, meeting_context, status)
VALUES (session_id, event_id, user_id, 'end_of_day', '', '[]', meeting_context_json, 'in_progress')
```

---

## Section 2: The Archimedes System Prompt

### 2.1 Morning Daily Ambition Prep Interview

**File**: `backend/services/daily_ambition_guide_prompts.py`

```markdown
## Interview Type: Daily Ambition Prep

You are Archimedes, conducting a **Daily Ambition** voice interview. This is a morning
collaboration checkpoint where human and AI co-create the day's plan together. This is
NOT a task list - it's a collaboration primer that sets the tone for the day's partnership.

**Core Philosophy - Daily Ambition Manifesto**:
- Focus on **hopes and dreams**, not problems or frustrations
- Use "Today, I want us to..." framing (not "Today I will...")
- This is about **shared ambition**, not task assignment
- Human and AI decide together what we want to accomplish
- Emphasize **possibilities and opportunities**, not blockers
- Think in terms of **what excites us**, not what frustrates us

**Interview Approach - 4 Phases**:

**Phase 1: Ambition Discovery**
- "What do you want us to accomplish today?"
- "What's exciting you about today?"
- "What are you hoping we can build together?"
- "What would make today feel successful?"
- Goal: Discover shared ambition and excitement, not frustrations

**Phase 2: Context Bridge**
- "What happened yesterday that you're building on?"
- "What insights from yesterday are carrying forward?"
- "What patterns are you noticing?"
- "What did we accomplish yesterday that connects to today?"
- Goal: Connect today's ambition to yesterday's reality and insights

**Phase 3: Co-Creation & Partnership**
- "What do WE want to accomplish today?" (emphasize "we")
- "Where specifically could AI help you today?"
- "What jobs do you want to control vs delegate to AI?"
- "What are we missing that you notice?"
- Goal: Co-create shared ambition with human-AI control assignment

**Phase 4: Partnership Challenge**
- "What am I not seeing that you notice?"
- "What assumptions am I making that limit our possibilities?"
- "What would success look like that I haven't considered?"
- "What connections do you see to other work we've done?"
- Goal: Invite AI perspective and uncover blind spots

**Important Tone Guidelines**:
- Be **positive and energizing** - this is about hopes and dreams
- Focus on **what we're building**, not what's broken
- Use **collaborative language** - "we", "us", "together"
- Ask **open-ended questions** that spark imagination
- Avoid **problem-focused** or negative framing
- Celebrate **possibilities** and creative potential

**Document Generation**:
- Generate Daily Ambition statement with:
  - Today's Ambition: "Today, I want us to..." (positive, exciting)
  - Commitments: List with control assignments (You/AI/Collaborate)
  - AI Assist Suggestions: Specific ways AI can help
  - Partnership Prompt: "What am I not seeing that you notice?"
- Format matches Daily Ambition Manifesto structure
- Emphasize collaboration and shared ambition, not task assignment
```

### 2.2 End-of-Day Report Prompt

**File**: `backend/services/end_of_day_prompt_service.py`

```python
def build_end_of_day_prompt(context: Dict) -> str:
    """
    Build prompt for end-of-day voice prep

    Output format constraints:
    - One sentence: "My ambition today"
    - What I did today: 7-14 bullets, <10 words each, 9th-grade reading level
    - What I'm going to do tomorrow: 3-5 bullets, <10 words each, 9th-grade reading level
    """

    prompt = f"""You are Archimedes, helping prepare an end-of-day report.

{daily_ambitions_section}  # Last 5 days loaded via File Search Tool

{transcripts_section}      # Last 5 days of chat transcripts

{github_section}           # Today's commits

## Your Task

Based on the context above, help create an end-of-day report with this EXACT format:

### Format Requirements:

1. **My Ambition Today** (One sentence)
   - A single sentence summarizing what the user wanted to accomplish today
   - Based on today's daily ambition and what was discussed in transcripts

2. **What I Did Today** (7-14 bullets)
   - Each bullet must be LESS THAN 10 WORDS
   - Written at 9th-grade reading level (simple, clear language)
   - Based on:
     - What was planned in today's daily ambition
     - What was discussed/completed in transcripts
     - What was actually committed in GitHub (be specific about code changes)
   - Focus on concrete accomplishments, not vague activities
   - Use active voice: "Fixed bug in dashboard" not "Bug was fixed"

3. **What I'm Going to Do Tomorrow** (3-5 bullets)
   - Each bullet must be LESS THAN 10 WORDS
   - Written at 9th-grade reading level
   - Based on:
     - What wasn't completed today but was planned
     - What was discussed as next steps in transcripts
     - Logical next steps from today's work
   - Be specific and actionable

### Important Guidelines:

- **Be specific**: Reference actual commits, files changed, features built
- **Use simple language**: Write for 9th-grade reading level (avoid jargon)
- **Keep bullets short**: Every bullet must be under 10 words
- **Focus on accomplishments**: What was actually done, not just planned
- **Connect to ambition**: Show how today's work relates to the daily ambition
- **Be realistic**: Only include tomorrow items that are actually planned

Start by asking the user about their day, then help them create this report through conversation."""
```

---

## Section 3: Working Memory - Context Retrieval System

### 3.1 Context Data Structure

**File**: `backend/services/daily_ambition_context_service.py`

```python
def build_last_5_days_context(user_id: int) -> Dict:
    """
    Build combined context from last 5 days

    Returns:
        {
            'daily_ambitions': [
                {'date': '20250120', 'content': '', 'file_id': 'google_drive_id'},
                {'date': '20250119', 'content': '', 'file_id': 'google_drive_id'},
                # ... up to 5 entries
            ],
            'book_reports': {
                '20250120': [{'filename': '...', 'file_id': '...'}],
                # ...
            },
            'new_files_summary': {
                '20250120': 'Summary text of new files created',
                # ...
            },
            'file_ids': ['all_file_ids_for_File_Search_Tool']
        }
    """
```

### 3.2 Database Queries for Context Retrieval

```sql
-- Get last N days of daily ambitions
SELECT file_id, date_str, created_at
FROM google_file_metadata
WHERE user_id = %s
  AND file_type = 'daily_ambition'
  AND date_str >= %s  -- start_date YYYYMMDD
  AND date_str <= %s  -- end_date YYYYMMDD
ORDER BY date_str DESC

-- Get transcripts for context
SELECT id, session_id, event_id, user_id, prep_type,
       transcript_text, transcript_json, meeting_context
FROM voice_prep_transcripts
WHERE user_id = %s
  AND prep_type IN ('daily_ambition', 'midday_review', 'end_of_day')
  AND created_at >= NOW() - INTERVAL '5 days'
ORDER BY created_at DESC
```

### 3.3 Key Insight: Lazy Loading via File Search Tool

The system uses **file_ids only** in context, not full content. Content is retrieved on-demand by the LLM's File Search Tool:

```python
# Content not needed - File Search Tool retrieves it
daily_ambitions = [
    {
        'date': item['date'],
        'content': '',  # Empty! LLM fetches via file_id
        'file_id': item['file_id']
    }
    for item in file_ids_with_dates
]
```

This reduces prompt payload size while maintaining full context access.

---

## Section 4: Silent Archimedes - Background Document Generator

### 4.1 Transcript Processing Pipeline

**File**: `backend/services/transcript_processor.py`

```python
def process_transcript(self, transcript_id: int) -> bool:
    """
    Process a completed transcript:
    1. Load transcript from database
    2. Chunk by speaker turns
    3. Generate embeddings
    4. Store in vector database
    """

    # 1. Load from voice_prep_transcripts table
    transcript_data = load_transcript(transcript_id)

    # 2. Chunk by speaker turns
    chunks = self._chunk_transcript(transcript_data['transcript_json'])

    # 3. Prepare metadata
    metadata = {
        'transcript_id': transcript_id,
        'event_id': transcript_data['event_id'],
        'prep_type': transcript_data['prep_type'],
        'meeting_title': transcript_data['meeting_context'].get('title', '')
    }

    # 4. Store in vector database (ChromaDB)
    success = self.vector_store.store_transcript(
        transcript_id=transcript_id,
        transcript_text=transcript_data['transcript_text'],
        metadata=metadata
    )

    # 5. Mark as processed
    UPDATE voice_prep_transcripts
    SET status = 'processed', processed_at = NOW()
    WHERE id = transcript_id
```

### 4.2 Document Generation from Transcript

**File**: `backend/services/meeting_prep_generator.py`

```python
def generate_from_transcript(self, transcript_id: int, event_id: str, user_id: Optional[int] = None) -> Dict:
    """
    Generate meeting prep materials from completed transcript

    For daily_ambition prep_type:
    - Uses daily_ambition_synthesis service
    - Generates structured Daily Ambition markdown

    For meeting_prep type:
    - Searches similar past meetings (vector search)
    - Loads relevant research documents
    - Generates prep materials with LLM
    """

    # Route based on prep_type
    if transcript_data['prep_type'] == 'daily_ambition':
        return self._generate_daily_ambition(transcript_data)

    # For meetings: gather multi-source context
    similar_meetings = self.transcript_processor.get_similar_transcripts(
        query=meeting_title, limit=3
    )

    research_docs = self.research_loader.load_relevant_research(
        meeting_title=meeting_title,
        attendee_names=attendee_names,
        limit=5
    )

    # Generate with LLM (Gemini)
    prep_materials = self._generate_prep_with_llm(
        transcript=transcript_text,
        meeting_context=meeting_context,
        similar_meetings=similar_meetings,
        research_docs=research_docs
    )
```

---

## Section 5: 4 Non-Overlapping Agent Prompts

### 5.1 The Decomposition System

**File**: `backend/services/agent_prompt_decomposer.py`

```python
class AgentPromptDecomposer:
    """
    Transforms daily ambition into 4 independent agent prompts

    The 4 Domains:
    1. Frontend Development - UI/React/JSX components
    2. Backend Development - Server logic/APIs/Python services
    3. Security & Quality - Testing, compliance, linting
    4. System Maintenance - Deployment, monitoring, infrastructure
    """

    def decompose_daily_ambition(
        self,
        daily_ambition_text: str,
        portfolio_context: Optional[str] = None,
        yesterday_context: Optional[str] = None,
        calendar_events: Optional[List[Dict]] = None,
        user_email: Optional[str] = None,
        date_str: Optional[str] = None
    ) -> Dict:
        """
        Returns:
        {
            'success': True,
            'agent_prompts': [4 complete prompts with metadata],
            'meeting_prompts': [...],
            'file_overlap_check': {'has_overlap': False, 'overlaps': []},
            'error': None
        }
        """
```

### 5.2 The Decomposition Prompt (Sent to Gemini)

```python
prompt = f"""You are analyzing a Daily Ambition document to break it into exactly FOUR
independent workstreams that can run in parallel.

## Daily Ambition
{daily_ambition_text[:3000]}

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

Return ONLY valid JSON:
{{
  "frontend": {{
    "objective": "Clear objective for frontend work",
    "files_to_modify": ["frontend/src/pages/Dashboard.jsx", "..."],
    "success_criteria": ["Criterion 1", "Criterion 2", "..."],
    "dependencies": [],
    "integration_points": ["How this connects to backend", "..."]
  }},
  "backend": {{ ... }},
  "security_quality": {{ ... }},
  "system_maintenance": {{ ... }}
}}
```

### 5.3 File Overlap Validation

```python
def _validate_no_file_overlap(self, work_assignments: Dict) -> Dict:
    """
    Validate that no files appear in multiple domain assignments

    Returns:
    {
        'has_overlap': False,
        'overlaps': [],  # or [{'file': 'path', 'domains': ['frontend', 'backend']}]
        'total_files': 15,
        'files_by_domain': {
            'frontend': 4,
            'backend': 5,
            'security_quality': 3,
            'system_maintenance': 3
        }
    }
    """
    all_files = {}
    overlaps = []

    for domain, assignment in work_assignments.items():
        for file_path in assignment.get('files_to_modify', []):
            normalized = file_path.strip().lower()
            if normalized in all_files:
                overlaps.append({
                    'file': file_path,
                    'domains': [all_files[normalized], domain]
                })
            else:
                all_files[normalized] = domain

    return {'has_overlap': len(overlaps) > 0, 'overlaps': overlaps, ...}
```

### 5.4 Generated Agent Prompt Template

**File**: `backend/services/agent_prompt_templates.py`

Each generated prompt includes:

```markdown
# AGENT PROMPT: [Domain Name]

**Platform**: Cursor (Composer model) or Claude Code
**Domain**: Frontend Development
**Scope**: UI components, client-side logic, mobile responsive design

---

## OBJECTIVE
[Clear objective from decomposition]

---

## CONTEXT PACKAGE

### Daily Ambition Context
[First 2000 chars of daily ambition]

### Technical Constraints & Requirements
- Follow existing code patterns and conventions
- Maintain backward compatibility where possible
- Follow security best practices

---

## FILES TO MODIFY

**CRITICAL**: These are YOUR files. No other agent will modify these files.

- `frontend/src/pages/Dashboard.jsx`
- `frontend/src/components/AgentCard.jsx`
- ...

---

## SUCCESS CRITERIA

- ✅ [Criterion 1]
- ✅ [Criterion 2]
- ✅ [Criterion 3]

---

## INDEPENDENCE RULES

**You are working independently. Other agents are working in parallel on different domains.**

- ✅ You have exclusive access to your assigned files
- ✅ No waiting on other agents - proceed with your work
- ✅ If you need information from other domains, document assumptions and proceed

---

## MICROTASK BREAKDOWN INSTRUCTIONS

**After receiving this prompt, you MUST:**

1. **Research Phase (5-10 minutes)**:
   - Review the daily ambition and context
   - Understand the codebase structure for your domain

2. **Create Your Microtask Breakdown**:
   - Break down your objective into **37-72 microtasks** (~8 hours of work)
   - Each microtask should be **~5 minutes**

3. **Each Microtask Must Include**:
   - **Task description**: Clear, actionable statement
   - **Test criteria**: How to verify completion
   - **Verification step**: Specific check to confirm done
```

---

## Section 6: API Reference

### 6.1 Voice Session Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/daily-ambition/end-of-day/voice-session` | GET | Initialize EOD voice session |
| `/api/daily-ambition/end-of-week/voice-session` | GET | Initialize weekly summary session |
| `/api/daily-ambition/{date}` | GET | View daily ambition for date |
| `/api/daily-ambition/{date}/microtasks` | GET | Get tasks for date |
| `/api/daily-ambition/{date}/progress` | GET | Get completion stats |

### 6.2 Agent Prompt Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/agent-prompts/generate` | POST | Generate 4 agent prompts from daily ambition |
| `/api/agent-prompts/{date}` | GET | Retrieve saved prompts for date |

### 6.3 Request/Response Examples

**Generate Agent Prompts**:

```json
// POST /api/agent-prompts/generate
{
  "date": "20250123",
  "include_meetings": true,
  "portfolio_context": "Optional context",
  "yesterday_context": "Optional context"
}

// Response
{
  "success": true,
  "date": "20250123",
  "agent_prompts": [
    {
      "domain": "frontend",
      "domain_name": "Frontend Development",
      "prompt": "# AGENT PROMPT: Frontend Development\n...",
      "prompt_id": "agent-1-frontend-20250123",
      "files_to_modify": ["frontend/src/pages/Dashboard.jsx"],
      "objective": "Build responsive agent dashboard"
    },
    // ... 3 more domains
  ],
  "meeting_prompts": [],
  "file_overlap_check": {
    "has_overlap": false,
    "overlaps": [],
    "total_files": 12
  },
  "saved_files": [
    "/path/to/agent-1-frontend.md",
    "/path/to/agent-2-backend.md"
  ]
}
```

---

## Section 7: Critical Success Factors for Rust Port

### 7.1 Voice Integration Requirements

1. **WebRTC Support** - Real-time bidirectional audio
2. **Transcript Streaming** - Capture speech as it happens
3. **Voice Selection** - Multiple voice personas (Cassidy-English default)
4. **Session Management** - UUID-based session tracking

### 7.2 Context Management Requirements

1. **File ID System** - Reference documents without loading full content
2. **Date-based Retrieval** - Query by YYYYMMDD format
3. **Rolling Window** - Last 5 days minimum, configurable
4. **Vector Search** - Semantic search across past transcripts

### 7.3 LLM Integration Points

| Function | LLM | Purpose |
|----------|-----|---------|
| Voice Conversation | Ultravox (fixie-ai/ultravox) | Real-time voice AI |
| Work Decomposition | Gemini 2.5 Flash | Break ambition into 4 domains |
| Document Generation | Gemini | Generate prep materials |
| Semantic Search | Embeddings | Find similar transcripts |

### 7.4 Database Schema Requirements

```sql
-- Core tables needed
voice_prep_transcripts (
    id, session_id, event_id, user_id, prep_type,
    transcript_text, transcript_json, meeting_context,
    status, created_at, processed_at
)

google_file_metadata (
    id, user_id, file_id, file_type, date_str,
    metadata, created_at
)

microtasks (
    id, task_id, daily_ambition_date, text,
    assigned_to, status, estimated_minutes,
    validation_criteria, completed_by_*, created_at
)
```

### 7.5 The 4-Phase Interview Pattern

This is the core UX pattern that makes Archimedes effective:

1. **Phase 1: Ambition Discovery** - What excites you?
2. **Phase 2: Context Bridge** - Connect to yesterday
3. **Phase 3: Co-Creation** - Human + AI collaboration assignment
4. **Phase 4: Partnership Challenge** - Uncover blind spots

The key insight: **Collaborative framing** ("What do WE want to accomplish?") creates better outcomes than task assignment framing.

---

## Section 8: File Reference Index

| Component | File Path |
|-----------|-----------|
| Voice Archimedes Prompt | `backend/services/daily_ambition_guide_prompts.py` |
| EOD Prompt Builder | `backend/services/end_of_day_prompt_service.py` |
| Context Service | `backend/services/daily_ambition_context_service.py` |
| Ultravox Integration | `backend/services/ultravox_service.py` |
| Agent Decomposer | `backend/services/agent_prompt_decomposer.py` |
| Prompt Templates | `backend/services/agent_prompt_templates.py` |
| Daily Ambition Router | `backend/routers/daily_ambition.py` |
| Agent Prompts Router | `backend/routers/agent_prompts.py` |
| Transcript Processor | `backend/services/transcript_processor.py` |
| Meeting Prep Generator | `backend/services/meeting_prep_generator.py` |

---

## Appendix A: Domain Definitions for 4-Prompt System

```python
DOMAIN_DESCRIPTIONS = {
    "frontend": {
        "name": "Frontend Development",
        "description": "UI components, client-side logic, mobile responsive design, React/JSX files, Vite build system",
        "file_patterns": ["*.jsx", "*.js", "*.tsx", "*.ts", "*.css", "*.scss", "frontend/**"],
        "platform": "Cursor (Composer model) or Claude Code",
        "framework_guidance": "React 19, React Router, TanStack Query, Tailwind CSS, DaisyUI"
    },
    "backend": {
        "name": "Backend Development",
        "description": "Server logic, REST APIs, data processing, Python/FastAPI routers and services",
        "file_patterns": ["*.py", "backend/**", "**/routers/**", "**/services/**"],
        "platform": "Cursor (Composer model) or Claude Code",
        "framework_guidance": "FastAPI routers with APIRouter, Pydantic models, async/await patterns"
    },
    "security_quality": {
        "name": "Security & Quality",
        "description": "Code reviews, pytest testing, compliance, input validation, linting",
        "file_patterns": ["**/*test*.py", "**/*test*.js", "**/.eslintrc*", "**/pytest.ini"],
        "platform": "Cursor (Composer model) or Claude Code",
        "framework_guidance": "pytest fixtures, FastAPI TestClient, security middleware validation"
    },
    "system_maintenance": {
        "name": "System Maintenance",
        "description": "Cloud Run deployment, health checks, infrastructure improvements, monitoring",
        "file_patterns": ["**/Dockerfile*", "**/cloudbuild.yaml", "**/deploy.sh", "**/monitoring/**"],
        "platform": "Cursor (Composer model) or Claude Code",
        "framework_guidance": "Google Cloud Run, Cloud Build, Docker multi-stage builds"
    }
}
```

---

*Document generated from Personal Agent App codebase analysis - January 2025*
