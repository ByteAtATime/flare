# Documentation Structure

This document explains the organization of planning and documentation in the Flareup project.

## 📋 Active Documents (ROOT LEVEL)

### [ROADMAP.md](/ROADMAP.md) ⭐ PRIMARY
**The single source of truth for project planning.**

- **Purpose:** Comprehensive roadmap and status tracking
- **Updated:** Dec 24, 2025 (v0.1.1)
- **Contains:**
  - Current status (75% Raycast parity)
  - Recent wins and completed work
  - Remaining work prioritized by phase
  - Future enhancements
  - Strategic priorities
  - Known issues and limitations
  - Milestones and progress tracking

**Always consult ROADMAP.md first for:**
- What's next to work on
- Current project status
- Feature priorities
- Milestone tracking

## 📚 Archived Documents (docs/archive/)

### Purpose
Historical planning documents from the Dec 21-22, 2025 audit period. These documents informed the current ROADMAP but have been largely superseded.

### Contents
- **[README.md](./docs/archive/README.md)** - Explains archive organization
- **[TODO.md](./docs/archive/TODO.md)** - Detailed task breakdown (Dec 21 audit)
- **[FEATURE_IDEAS.md](./docs/archive/FEATURE_IDEAS.md)** - Feature brainstorming
- **[RAYCAST_GAPS.md](./docs/archive/RAYCAST_GAPS.md)** - Parity analysis
- **[AUDIT_REPORT.md](./docs/archive/AUDIT_REPORT.md)** - Comprehensive code audit
- **[CLAUDE_REVIEW_2025-12-22.md](./docs/archive/CLAUDE_REVIEW_2025-12-22.md)** - Code review

### When to Use
- **For historical context** - Understanding how we got here
- **For detailed technical debt** - AUDIT_REPORT.md has deep analysis
- **For ideation** - FEATURE_IDEAS.md has brainstorming notes

**Do NOT use for:**
- Current priorities (use ROADMAP.md instead)
- Feature planning (use ROADMAP.md instead)
- Status tracking (use ROADMAP.md instead)

## 🔄 Workflow

### Planning New Work
1. Check [ROADMAP.md](/ROADMAP.md) for current priorities
2. Identify next phase/task
3. Create implementation plan in brain artifacts
4. Execute and update ROADMAP.md as needed

### Updating Status
1. Update [ROADMAP.md](/ROADMAP.md) "Recent Wins" section
2. Update version numbers and parity percentage
3. Move completed items from "Remaining Work" to changelog
4. Adjust priorities based on new information

### Completing Milestones
1. Update [ROADMAP.md](/ROADMAP.md) milestone checklist
2. Update parity percentage
3. Add changelog entry
4. Consider archiving any temporary planning docs

## 📝 Documentation Principles

### ROADMAP.md Should Be:
- ✅ Comprehensive yet scannable
- ✅ Updated as work progresses
- ✅ Single source of truth
- ✅ Written for humans (clear, organized)

### Archived Docs Should Be:
- ✅ Clearly marked as archived
- ✅ Pointing to ROADMAP.md
- ✅ Preserved for historical reference
- ✅ NOT actively maintained

## 🎯 Quick Reference

| Need to... | Use this document |
|------------|-------------------|
| Know what to work on next | [ROADMAP.md](/ROADMAP.md) → "Remaining Work" |
| Check current status | [ROADMAP.md](/ROADMAP.md) → "Current Status" |
| See what's been done | [ROADMAP.md](/ROADMAP.md) → "Recent Wins" |
| Understand a past decision | [docs/archive/](/docs/archive/) |
| Plan future features | [ROADMAP.md](/ROADMAP.md) → "Future Enhancements" |
| Find technical debt | [docs/archive/AUDIT_REPORT.md](/docs/archive/AUDIT_REPORT.md) |

---

**Last Updated:** Dec 24, 2025  
**Maintained By:** Keep ROADMAP.md current, archive everything else
