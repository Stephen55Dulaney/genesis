# Voice Reflection Loop Implementation Guide
## Wheeler Research Hub - Voice Agent with Persistent Context Memory

This guide documents the complete implementation of the Voice Reflection Loop system built with Ultravox voice agents, including the two-layer personal/shared context architecture.

---

## Architecture Overview

```
                    ┌─────────────────────────────────────────────────────────┐
                    │                   Research Project                       │
                    │                                                          │
                    │   ┌──────────────────────────────────────────────────┐  │
                    │   │           SHARED WORLD MEMORY                     │  │
                    │   │      (research_projects.world_memory)             │  │
                    │   │                                                   │  │
                    │   │   • Project axioms, verified findings             │  │
                    │   │   • Team decisions and conclusions                │  │
                    │   │   • Visible to ALL collaborators                  │  │
                    │   │   • Updated via "Contribute to World Memory"      │  │
                    │   └──────────────────────────────────────────────────┘  │
                    │                                                          │
                    │   ┌─────────────────────┐    ┌─────────────────────┐    │
                    │   │  Stephen's Context   │    │   Jeff's Context    │    │
                    │   │  (user_project_      │    │  (user_project_     │    │
                    │   │   context)           │    │   context)          │    │
                    │   │                      │    │                     │    │
                    │   │  • My reflections    │    │  • My reflections   │    │
                    │   │  • My conversation   │    │  • My conversation  │    │
                    │   │    history           │    │    history          │    │
                    │   │  • PRIVATE to me     │    │  • PRIVATE to him   │    │
                    │   └─────────────────────┘    └─────────────────────┘    │
                    └─────────────────────────────────────────────────────────┘
```

---

## System Components

### 1. Database Tables

#### `user_project_context` - Personal Context Layer
```sql
CREATE TABLE public.user_project_context (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL,
  project_id UUID NOT NULL REFERENCES public.research_projects(id) ON DELETE CASCADE,
  personal_notes TEXT DEFAULT '',
  conversation_summary TEXT DEFAULT '',
  last_conversation_date TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT now(),
  updated_at TIMESTAMPTZ DEFAULT now(),
  UNIQUE(user_id, project_id)
);

-- RLS: Users can only access their own context
ALTER TABLE public.user_project_context ENABLE ROW LEVEL SECURITY;
CREATE POLICY "Users can view their own project context" 
  ON public.user_project_context FOR SELECT USING (auth.uid() = user_id);
CREATE POLICY "Users can insert their own project context" 
  ON public.user_project_context FOR INSERT WITH CHECK (auth.uid() = user_id);
CREATE POLICY "Users can update their own project context" 
  ON public.user_project_context FOR UPDATE USING (auth.uid() = user_id);
```

#### `conversation_transcripts` - Real-time Transcript Storage
```sql
-- Stores individual turns during voice calls
id UUID, project_id UUID, user_id UUID, session_id TEXT, role TEXT, message TEXT, created_at TIMESTAMPTZ
```

#### `reflection_sessions` - AI-Generated Insights
```sql
-- Stores structured reflection output
id UUID, project_id UUID, user_id UUID, session_id TEXT, 
reflection_type TEXT, insights JSONB, summary TEXT, 
key_themes TEXT[], action_items TEXT[], emotional_tone TEXT
```

---

### 2. Voice Agent Edge Function (`dr-wheeler-voice`)

**Purpose**: Creates Ultravox voice calls with full project context injected into the system prompt.

**Key Features**:
- Fetches shared `world_memory` from `research_projects`
- Fetches personal `personal_notes` from `user_project_context` using `userId`
- Passes both layers to the prompt builder
- Returns `joinUrl` for Ultravox WebSocket connection

```typescript
// Fetch personal context for this specific user
async function fetchPersonalContext(supabase, userId, projectId) {
  const { data } = await supabase
    .from('user_project_context')
    .select('personal_notes, last_conversation_date')
    .eq('project_id', projectId)
    .eq('user_id', userId)
    .maybeSingle();
    
  if (data?.personal_notes) {
    return `### Previous Conversation Notes (Private to you)
${data.personal_notes}

Last conversation: ${data.last_conversation_date || 'First conversation'}`;
  }
  return '';
}
```

---

### 3. Transcript Capture Hook (`useConversationCapture.ts`)

**Purpose**: Real-time capture of voice conversation turns during Ultravox calls.

**Key Pattern - Preventing Race Conditions**:
```typescript
// Refs are used to avoid stale-closure issues when Ultravox events fire
const sessionIdRef = useRef<string | null>(null);
const isRecordingRef = useRef(false);
const processedOrdinalsRef = useRef<Set<number>>(new Set()); // Prevent duplicates

// IMPORTANT: Update refs SYNCHRONOUSLY before async operations
const startConversation = async (input) => {
  sessionIdRef.current = newSessionId;      // Sync first!
  isRecordingRef.current = true;
  // ... then update React state
};
```

**Ultravox v0.5.0 Transcript Access**:
```typescript
session.addEventListener('transcripts', async () => {
  const transcripts = session.transcripts || [];
  
  for (const t of transcripts) {
    // Skip already-processed transcripts using ordinal
    const ordinal = t.ordinal;
    if (ordinal !== undefined && processedOrdinalsRef.current.has(ordinal)) {
      continue;
    }
    
    // Only process final transcripts
    if (t.text && t.isFinal) {
      processedOrdinalsRef.current.add(ordinal);
      await addTurn({
        speaker: t.speaker === 'user' ? 'user' : 'ai',
        text: t.text
      });
    }
  }
});
```

---

### 4. Reflection Edge Function (`reflect-conversation`)

**Purpose**: Analyzes completed conversations and generates structured insights.

**Data Flow**:
1. Fetch all `conversation_transcripts` for the `sessionId`
2. Send to AI (Lovable AI Gateway with `google/gemini-2.5-flash`)
3. Parse structured JSON response
4. Save to `reflection_sessions` table
5. **CRITICAL**: Append insights to `user_project_context.personal_notes` (NOT shared world_memory)

**AI Response Structure**:
```json
{
  "summary": "2-4 sentence summary",
  "key_themes": ["theme1", "theme2"],
  "action_items": ["next step 1", "next step 2"],
  "emotional_tone": "curious",
  "insights": [
    {
      "type": "discovery | question | connection | hypothesis",
      "content": "specific insight",
      "confidence": 0.85
    }
  ]
}
```

**Personal Context Update Logic**:
```typescript
// Build update for personal notes (NOT world memory)
const personalUpdate = `
## Voice Conversation Insights (${date})

**Summary**: ${reflection.summary}
**Key Themes**: ${reflection.key_themes.join(', ')}

**High-Confidence Insights**:
${insights.filter(i => i.confidence >= 0.6).map(i => `- [${i.type}] ${i.content}`).join('\n')}
---
`;

// Upsert to personal context
await supabase.from('user_project_context').upsert({
  user_id: userId,
  project_id: projectId,
  personal_notes: existingNotes + personalUpdate,
  last_conversation_date: new Date().toISOString()
}, { onConflict: 'user_id,project_id' });
```

---

### 5. Prompt Architecture (`_shared/prompts.ts`)

**Two-Layer Context Injection**:
```typescript
export function buildDrWheelerPrompt(
  userName: string,
  researchTopic: string,
  projectContext?: string,    // Shared world memory
  personalContext?: string    // User's private notes
): string {
  return `...
${projectContext ? `### SHARED PROJECT CONTEXT (Visible to all collaborators):
${projectContext}
` : ''}
${personalContext ? `### YOUR PERSONAL NOTES (From previous conversations - private to you):
${personalContext}
` : ''}
...`;
}
```

---

### 6. Frontend Components

#### `DrWheelerVoice.tsx` - Main Voice Interface
- Manages Ultravox session lifecycle
- Passes `currentUserId` to edge function for personal context
- Handles auto-reflection toggle
- Shows capture status badges (turns captured, etc.)

#### `WorldMemoryViewer.tsx` - Context Display
- Two tabs: "Shared World Memory" and "My Personal Notes"
- Edit and clear functionality for personal notes
- Token count estimation for context size awareness

#### `ContributeToWorldMemory.tsx` - Deliberate Sharing
- Review personal notes before sharing
- Write curated contribution to shared world memory
- Attribution with user name and date

---

## Data Flow Diagram

```
┌──────────────────┐     ┌─────────────────────┐     ┌──────────────────────┐
│   User starts    │────▶│  dr-wheeler-voice   │────▶│   Ultravox API       │
│   voice call     │     │  edge function      │     │   creates call       │
└──────────────────┘     │                     │     └──────────────────────┘
                         │  Fetches:           │              │
                         │  • world_memory     │              ▼
                         │  • personal_notes   │     ┌──────────────────────┐
                         │  (for this user)    │     │  WebSocket connection│
                         └─────────────────────┘     │  to voice agent      │
                                                     └──────────────────────┘
                                                              │
                         ┌─────────────────────┐              │
                         │  useConversation-   │◀─────────────┘
                         │  Capture hook       │  Transcript events
                         │                     │
                         │  Saves each turn to │
                         │  conversation_      │
                         │  transcripts table  │
                         └─────────────────────┘
                                  │
                         ┌────────▼────────────┐
                         │  Call ends          │
                         │  Auto-reflect       │
                         │  triggered          │
                         └────────┬────────────┘
                                  │
                         ┌────────▼────────────┐
                         │ reflect-conversation│
                         │ edge function       │
                         │                     │
                         │ 1. Fetch transcripts│
                         │ 2. AI analysis      │
                         │ 3. Save reflection  │
                         │ 4. Update personal_ │
                         │    notes (private)  │
                         └─────────────────────┘
```

---

## Key Implementation Details

### Preventing World Memory Contamination
The reflection loop writes to `user_project_context.personal_notes` instead of `research_projects.world_memory`. This ensures:
- Jeff's conversation insights don't pollute Stephen's context
- Each user has their own conversation continuity
- Shared world memory stays clean with only deliberately contributed content

### User Identification in Voice Calls
```typescript
// In DrWheelerVoice.tsx
const { data: { user } } = await supabase.auth.getUser();
setCurrentUserId(user.id);

// Pass to edge function
await supabase.functions.invoke('dr-wheeler-voice', {
  body: {
    userId: currentUserId,  // This identifies the researcher
    projectId,
    userName,
    researchTopic
  }
});
```

### Ultravox SDK Version
- Using `ultravox-client` v0.5.0
- Access transcripts via `session.transcripts` array
- Filter for `isFinal` transcripts to avoid duplicates
- Enable `transcript` and `state` data messages in call config

---

## Dependencies

```json
{
  "ultravox-client": "^0.5.0",
  "@supabase/supabase-js": "^2.86.0",
  "framer-motion": "^12.23.24"
}
```

---

## Environment Variables Required

- `ULTRAVOX_API_KEY` - For Ultravox voice calls
- `LOVABLE_API_KEY` - For AI Gateway (reflection analysis)
- Standard Supabase connection vars (auto-configured in Lovable Cloud)

---

## Replicating in QuantumDynamX.com

1. **Create the `user_project_context` table** with RLS policies
2. **Update your voice edge function** to fetch both shared and personal context
3. **Update your prompt builder** to accept `personalContext` parameter
4. **Modify the reflection function** to write to `personal_notes` instead of shared memory
5. **Add the WorldMemoryViewer** with tabs for shared/personal
6. **Add ContributeToWorldMemory** for deliberate sharing

The key architectural insight: **Reflections are private by default, sharing is deliberate**.
