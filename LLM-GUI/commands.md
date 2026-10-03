# Command ids

For `pdfcer-remote run <id>`, `PDFCER_DIAG_INVOKE`, and the `ribbon.item.<id>` ui-rect names.
Format: `id  Label`. The live list, with each id's current state, is `pdfcer-remote commands`.
A command is hidden (not greyed) in a mode that lacks its ribbon tab. Ids on no main tab are offered in every mode.
Clipboard ids (`edit.copy`, `edit.cut`, `edit.paste`...) decide for themselves what the mode allows.
`format.*` ids live on the contextual Format tab: they need a selection and are disabled without one.
A label ending `…` opens a dialog on the user's screen.
This list is the full build. `pdfcer-remote commands` shows what this build actually registered.

## Every mode (Read, Review, Edit)

edit.copy  Copy
edit.cut  Cut
edit.duplicate  Duplicate
edit.find  Find
edit.paste  Paste
edit.paste_duplicate  Paste as duplicate
edit.redo  Redo
edit.undo  Undo

file.about  About pdfcer
file.add_archive_timestamp  Add archive time-stamp…
file.add_validation_evidence  Add validation evidence…
file.close  Close
file.copy_document_text  Copy document text
file.copy_page_text  Copy page text
file.document_properties  Document properties
file.encrypt  Encrypt…
file.export_dxf  Export DXF…
file.export_form_data  Export form data…
file.export_image  Export image…
file.export_tables  Export tables…
file.export_text  Export text…
file.export_word  Word document…
file.fonts  Fonts
file.import_form_data  Import form data…
file.import_text  Import text as pages…
file.new  New
file.new_from_template  New from template…
file.ocr  Recognise text…
file.open  Open…
file.open_in_acrobat  Acrobat
file.permissions  Permissions…
file.print  Print…
file.properties  Properties
file.purge_password_values  Remove old passwords…
file.remove_ocr  Remove OCR text
file.save  Save
file.save_as  Save as…
file.save_compacted  Save a compacted copy…
file.save_copy  Save a copy…
file.settings  Settings…
file.shortcuts  Keyboard shortcuts
file.sign  Sign…
file.stamp_collection  Save as stamp collection

format.arrowheads  Arrowheads
format.bold  Bold
format.colour  Line colour
format.delete  Delete
format.dimension_diameter  Show as diameter
format.dimension_radius  Show as radius
format.fill  Fill colour
format.font  Font
format.font_colour  Colour
format.font_size  Size
format.italic  Italic
format.line_style  Line style
format.line_width  Line width
format.merge_text_runs  Merge text runs
format.opacity  Opacity
format.properties  Properties
format.select_form  Select the form
format.select_text_line  Select this line of text
format.unshare_form  Give this page its own copy

markup.add_node  Add a point here
markup.flatten  Make part of the page
markup.remove_node  Remove this point

mode.edit  Edit
mode.read  Read
mode.review  Review

view.close_other_documents  Close others
view.dock_all_panels  Dock all panels
view.fullscreen  Full screen
view.grid  Grid
view.guides  Guides
view.line_weights  Line weights
view.next_document  Next document
view.ocr_layer  OCR text
view.off_page  Off-page content
view.page_continuous  Continuous
view.page_facing  Facing
view.page_facing_continuous  Facing continuous
view.page_single  Single page
view.panel_bookmarks  Bookmarks
view.panel_close  Close panel
view.panel_dock  Dock panel
view.panel_float  Float panel
view.panel_forms  Fill form
view.panel_layers  Layers
view.panel_objects  Objects
view.panel_pages  Pages
view.panel_signatures  Signatures
view.previous_document  Previous document
view.rail_auto_hide  Auto-hide left strip
view.read_mode  Read mode
view.reset_layout  Reset layout
view.ribbon_auto_hide  Auto-hide ribbon
view.rulers  Rulers
view.show_annotations  Annotations
view.show_points  Points
view.smart_select  Smart select
view.text_chunks  Text chunks
view.tool_hand  Hand
view.tool_node  Points
view.tool_select  Select
view.tool_snapshot  Snapshot
view.tool_text  Text
view.zoom_actual  Actual size
view.zoom_fit_height  Fit height
view.zoom_fit_page  Fit page
view.zoom_fit_width  Fit width
view.zoom_region  Zoom to region
view.zoom_selection  Zoom to selection

## Review and Edit (hidden in Read)

markup.arrow  Arrow
markup.bring_forward  Bring forward
markup.bring_to_front  Bring to front
markup.cloud  Revision cloud
markup.comments  Comments
markup.ellipse  Ellipse
markup.finish  Finish shape
markup.flatten_page  Make all part of the page
markup.highlight  Highlight
markup.ink  Freehand
markup.polygon  Polygon
markup.polyline  Polyline
markup.rectangle  Rectangle
markup.send_backward  Send backward
markup.send_to_back  Send to back
markup.squiggly  Squiggly
markup.stamp  Stamp
markup.sticky_note  Sticky note
markup.strikeout  Strikeout
markup.text_box  Text box
markup.underline  Underline

measure.finish  Finish
measure.length  Length
measure.linear  Linear
measure.manage_groups  Dimension groups
measure.perimeter  Perimeter
measure.radius_diameter  Radius / diameter
measure.set_scale  Set scale
measure.two_line  Two-line

pages.bates  Bates numbering…
pages.bates_remove  Remove Bates numbers
pages.copy  Copy pages
pages.crop  Crop…
pages.cut  Cut pages
pages.delete  Delete pages
pages.extract  Extract…
pages.insert_from_file  Insert from file…
pages.labels  Number pages…
pages.merge_into  Merge into this document…
pages.move_down  Move down
pages.move_up  Move up
pages.paste  Paste pages
pages.resize  Sheet size…
pages.rotate_left  Rotate left
pages.rotate_right  Rotate right

## Edit only

edit.add_text  Add text
edit.align  Align and Distribute
edit.align_bottom  Align bottom edges
edit.align_centre  Centre on both axes
edit.align_centre_x  Centre on vertical axis
edit.align_centre_y  Centre on horizontal axis
edit.align_left  Align left edges
edit.align_right  Align right edges
edit.align_top  Align top edges
edit.attachments  Attachments
edit.copy_as_vector  Copy as vector
edit.form_check_box  Check box
edit.form_choice  Drop-down
edit.form_flatten  Flatten
edit.form_manage_fields  Manage fields
edit.form_push_button  Button
edit.form_radio_button  Radio button
edit.form_repair_fonts  Repair fonts
edit.form_text_field  Text field
edit.insert_3d  3D model…
edit.insert_image  Image…
edit.offpage  Check for content off the sheet
edit.redact  Redact
edit.redact_apply  Apply redactions
edit.redact_selection  Redact selection
edit.reflow_block  Reflow paragraph
edit.select_all  Select all
edit.text  Edit text

tools.embed_fonts  Embed fonts
tools.font_folders  Font folders…
tools.merge_files  Merge files…
tools.render_diagnostics  Render diagnostics
tools.unembed_fonts  Unembed fonts
