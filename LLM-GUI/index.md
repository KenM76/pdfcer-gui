# pdfcer-gui: help for an LLM agent

pdfcer-gui is a Windows PDF reader and editor for drawings and documents. These
files tell an agent how to inspect, change and verify PDFs through it, without
breaking the user's open work. Read only the file the task needs.

## Start here: change the document the user has open

1. `pdfcer-remote list` finds the window. Then `pdfcer-remote --pid N --purpose "<why>" state`.
2. `render` and read the PNG; `objects` lists what is on the page.
3. `select i,j`, then `run <command id>` (`run mode.edit` first if it says `hidden`).
4. `render` again to verify. Leave saving to the user.
5. If there is no live link, follow the hand-off in file-safety.md, then cli.md.

## Files

| File | Read it when |
|---|---|
| program.md | You need to know what ships, the modes, the ribbon tabs, panels, and canvas behaviour |
| file-safety.md | Before writing any file the user might have open |
| live-link.md | Driving the user's open window with `pdfcer-remote.exe` (verbs, permission, protocol) |
| commands.md | You need a command id for `run`, `PDFCER_DIAG_INVOKE` or a `ribbon.item.` rect |
| shortcuts.md | You need a keyboard chord, or the user asks how to do something by key |
| cli.md | Editing a file on disk with `pdfcer.exe`: conventions, save modes, exit codes, task map |
| cli-subcommands.md | Looking for the subcommand that does X (all 202, grouped) |
| markup-recipe.md | The user wants markup instructions on a drawing carried out |
| hidden-driving.md | No CLI subcommand fits and the GUI must be driven privately, off-screen |
| diag-env.md | The environment variables for hidden driving, and the file-picker answers |
| pointer-script.md | Scripted clicks, drags, keys, text and screenshots for a hidden copy |
| userdata.md | Where settings, layout and recent files live, with every preference key |
| troubleshooting.md | Something refused, hung, vanished or did nothing |

## Rules that apply everywhere

- Never write a file the user has open; a change written from outside is lost at
  their next Save. Use the live link, or ask them to save and close first.
- Never drive the user's own `pdfcer-gui.exe` with diagnostic variables. Drive a
  copy in a scratch folder: a process writes `userdata\` beside its exe.
- End only processes you started, by PID.
- A ce dimension is a measurement pdfcer authored (the `dimension-*` commands).
  A pdf dimension is CAD-exported vectors and text on the page, edited as ordinary
  objects. Never use the word without one of these prefixes.
- Ask before anything irreversible: `redact-apply`, a full save of a signed file,
  `print --send`.
