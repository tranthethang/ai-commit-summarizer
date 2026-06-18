---
name: create_skill
description: Automates the creation of professional-grade Antigravity skills. Incorporates best practices for progressive disclosure, resource organization, and automatic agent registration.
---

# Create Skill

Use this skill to define a new capability for Antigravity. It enforces a professional structure inspired by industry best practices (conciseness, progressive disclosure, and modularity).

## Inputs
- **Skill Name**: The name of the skill (snake_case).
- **Description**: A short, action-oriented summary of what the skill does.
- **Instructions**: The core logic or checklist for the skill.

## Tooling Strategy
- Use `write_to_file` to create SKILL.md and related files.
- Use `run_command` to verify directory creation if not using `write_to_file`'s auto-create.

## Workflow
1. **Design & Strategy**: Before creating files, ask the user (or deduce from context):
   - *Trigger*: What specifically should trigger this skill? (This goes into the description).
   - *Complexity*: 
     - Simple: Just a SKILL.md checklist.
     - Complex: Needs `scripts/` (for tools) or `references/` (for docs).

2. **Create Skill Structure**: Create the directory at `~/.gemini/antigravity/skills/[skill_name]/`. Based on Complexity, create subfolders if needed:
   - `[skill_name]/scripts/` (Executable code)
   - `[skill_name]/references/` (Documentation)
   - `[skill_name]/assets/` (Templates/Images)

3. **Write SKILL.md**: (The "Concise" Rule)
   - Frontmatter layout must match standard YAML with `name` and `description`.
   - Keep instructions under 500 words (progressive disclosure).

4. **Register Slash Command (Workflow)**: Create the workflow trigger file at:
   - `~/.gemini/antigravity/skills/.agent/workflows/[skill_name].md`
   - Content inside:
```markdown
     ---
     description: [Short, action-oriented summary]
     ---
     1. Run the [skill_name] skill.
     ```

5. **Update README.md**: Append the new skill into the global `README.md` catalog under the correct functional category.

6. **Final Confirmation**: Confirm to the user that the structure is ready and the `/[skill_name]` command is active.