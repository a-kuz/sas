use super::tools::EditorTool;
use super::tools::SelectedObject;
use macroquad::prelude::*;

pub struct HelpPanel {
    scroll_offset: f32,
    max_scroll: f32,
}

impl HelpPanel {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0.0,
            max_scroll: 0.0,
        }
    }

    pub fn render(
        &mut self,
        current_tool: EditorTool,
        selected_object: &Option<SelectedObject>,
        map_name: &str,
        zoom: f32,
        ambient_light: f32,
    ) {
        let panel_width = 800.0;
        let panel_height = 700.0;
        let panel_x = screen_width() / 2.0 - panel_width / 2.0;
        let panel_y = screen_height() / 2.0 - panel_height / 2.0;

        draw_rectangle(
            panel_x,
            panel_y,
            panel_width,
            panel_height,
            Color::from_rgba(0, 0, 0, 240),
        );
        draw_rectangle_lines(panel_x, panel_y, panel_width, panel_height, 2.0, WHITE);

        let mouse_wheel_y = mouse_wheel().1;
        if mouse_wheel_y != 0.0 {
            self.scroll_offset -= mouse_wheel_y * 20.0;
        }

        let mut y = panel_y + 30.0 - self.scroll_offset;
        let line_height = 20.0;
        let content_start_y = panel_y + 60.0;
        let content_end_y = panel_y + panel_height - 20.0;

        draw_text(
            "MAP EDITOR HELP",
            panel_x + 250.0,
            panel_y + 30.0,
            24.0,
            YELLOW,
        );
        draw_text(
            &format!(
                "Map: {} | Zoom: {:.1}x | Ambient: {:.2}",
                map_name, zoom, ambient_light
            ),
            panel_x + 20.0,
            panel_y + 55.0,
            16.0,
            GRAY,
        );

        y = content_start_y - self.scroll_offset;

        let sections = self.get_help_sections(current_tool, selected_object);

        for section in &sections {
            if y > content_start_y - 30.0 && y < content_end_y {
                draw_text(&section.title, panel_x + 20.0, y, 20.0, YELLOW);
            }
            y += line_height + 5.0;

            for line in &section.lines {
                if y > content_start_y - 30.0 && y < content_end_y {
                    let color = if line.is_empty() { WHITE } else { WHITE };
                    draw_text(line, panel_x + 40.0, y, 17.0, color);
                }
                y += line_height;
            }

            y += 10.0;
        }

        self.max_scroll = (y - content_start_y - panel_height + 100.0).max(0.0);
        self.scroll_offset = self.scroll_offset.max(0.0).min(self.max_scroll);

        if self.max_scroll > 0.0 {
            let scrollbar_height = panel_height - 100.0;
            let scrollbar_x = panel_x + panel_width - 15.0;
            let scrollbar_y = panel_y + 60.0;

            draw_rectangle(
                scrollbar_x,
                scrollbar_y,
                8.0,
                scrollbar_height,
                Color::from_rgba(100, 100, 100, 100),
            );

            let thumb_height =
                (scrollbar_height * (panel_height / (y - content_start_y))).max(20.0);
            let thumb_y = scrollbar_y
                + (self.scroll_offset / self.max_scroll) * (scrollbar_height - thumb_height);

            draw_rectangle(
                scrollbar_x,
                thumb_y,
                8.0,
                thumb_height,
                Color::from_rgba(200, 200, 200, 200),
            );
        }

        draw_text(
            "Press H or F1 to close | Scroll to navigate",
            panel_x + 20.0,
            panel_y + panel_height - 10.0,
            16.0,
            GRAY,
        );
    }

    fn get_help_sections(
        &self,
        current_tool: EditorTool,
        selected_object: &Option<SelectedObject>,
    ) -> Vec<HelpSection> {
        let mut sections = vec![];

        sections.push(HelpSection {
            title: "GLOBAL CONTROLS (Always Available)".to_string(),
            lines: vec![
                "0-8: Switch tools (0=Select, 1=Draw, 2=Erase, 3=Spawn, 4=Item, 5=Jump, 6=Teleport, 7=Light, 8=BG)".to_string(),
                "WASD: Move camera".to_string(),
                "G: Toggle grid".to_string(),
                "P: Toggle properties panel".to_string(),
                "H / F1: Toggle this help".to_string(),
                "Ctrl+S: Save map".to_string(),
                "Ctrl+Scroll: Zoom in/out".to_string(),
                "Ctrl + +/-: Zoom in/out".to_string(),
                "Ctrl+0: Reset zoom to 1.0x".to_string(),
                "ESC: Close dialogs / Reset to Draw tool".to_string(),
                "".to_string(),
            ],
        });

        match current_tool {
            EditorTool::Draw => {
                sections.push(HelpSection {
                    title: "DRAW TOOL (Current)".to_string(),
                    lines: vec![
                        "LMB: Paint tiles with current texture".to_string(),
                        "Shift+LMB Drag: Draw straight line".to_string(),
                        "Q: Cycle brush size (1x1 -> 2x2 -> 4x4)".to_string(),
                        "T: Next texture".to_string(),
                        "Ctrl+T: Open texture picker".to_string(),
                        "X: Toggle shader on/off".to_string(),
                        "Ctrl+X: Open shader picker".to_string(),
                        "".to_string(),
                        "Texture Scaling:".to_string(),
                        "  Shift+Scroll: Change texture scale".to_string(),
                        "  Shift+[/]: Decrease/increase scale".to_string(),
                        "  Range: 0.1x - 10.0x".to_string(),
                        "".to_string(),
                        "Texture Offset:".to_string(),
                        "  Cmd+Arrows: Move offset (1px)".to_string(),
                        "  Cmd+Shift+Arrows: Fine offset (0.1px)".to_string(),
                        "  Cmd+0: Reset offset to (0,0)".to_string(),
                        "".to_string(),
                    ],
                });
            }
            EditorTool::Erase => {
                sections.push(HelpSection {
                    title: "ERASE TOOL (Current)".to_string(),
                    lines: vec![
                        "LMB: Erase tiles".to_string(),
                        "Shift+LMB Drag: Erase in straight line".to_string(),
                        "Q: Cycle brush size (1x1 -> 2x2 -> 4x4)".to_string(),
                        "".to_string(),
                    ],
                });
            }
            EditorTool::Light => {
                sections.push(HelpSection {
                    title: "LIGHT TOOL (Current)".to_string(),
                    lines: vec![
                        "LMB: Place new light".to_string(),
                        "Scroll (hover): Adjust light radius".to_string(),
                        "T (hover): Cycle radius presets".to_string(),
                        "F (hover): Toggle flicker effect".to_string(),
                        "+/- (hover): Adjust intensity".to_string(),
                        "".to_string(),
                        "When light selected:".to_string(),
                        "  Arrows: Move light (4px)".to_string(),
                        "  +/-: Adjust intensity".to_string(),
                        "  [/]: Adjust radius".to_string(),
                        "  Delete: Remove light".to_string(),
                        "".to_string(),
                    ],
                });
            }
            EditorTool::Background => {
                sections.push(HelpSection {
                    title: "BACKGROUND TOOL (Current)".to_string(),
                    lines: vec![
                        "LMB: Place background element".to_string(),
                        "B: Next background texture".to_string(),
                        "Ctrl+B: Open background texture picker".to_string(),
                        "A: Toggle additive blend mode".to_string(),
                        "V: Toggle snap to grid".to_string(),
                        "+/-: Change scale".to_string(),
                        "[/]: Change alpha (transparency)".to_string(),
                        "".to_string(),
                    ],
                });
            }
            EditorTool::Item => {
                sections.push(HelpSection {
                    title: "ITEM TOOL (Current)".to_string(),
                    lines: vec![
                        "LMB: Place item".to_string(),
                        "I: Change item type".to_string(),
                        "".to_string(),
                    ],
                });
            }
            EditorTool::JumpPad => {
                sections.push(HelpSection {
                    title: "JUMP PAD TOOL (Current)".to_string(),
                    lines: vec![
                        "LMB: Place jump pad".to_string(),
                        "When selected:".to_string(),
                        "  +/-: Adjust force Y".to_string(),
                        "  [/]: Adjust force X".to_string(),
                        "".to_string(),
                    ],
                });
            }
            EditorTool::Teleporter => {
                sections.push(HelpSection {
                    title: "TELEPORTER TOOL (Current)".to_string(),
                    lines: vec![
                        "LMB: Place teleporter".to_string(),
                        "When selected:".to_string(),
                        "  Arrows: Move (4px)".to_string(),
                        "  Shift+Arrows: Resize".to_string(),
                        "  R: Set destination (then click)".to_string(),
                        "".to_string(),
                    ],
                });
            }
            _ => {}
        }

        if selected_object.is_some() {
            sections.push(HelpSection {
                title: "OBJECT SELECTED".to_string(),
                lines: vec![
                    "Arrows: Move object (4px step)".to_string(),
                    "Delete / Backspace: Delete object".to_string(),
                    "".to_string(),
                ],
            });
        }

        sections.push(HelpSection {
            title: "NAVIGATION GRAPH".to_string(),
            lines: vec![
                "N: Toggle nav graph visualization".to_string(),
                "E: Toggle edges (when nav visible)".to_string(),
                "Ctrl+N: Generate nav graph".to_string(),
                "Shift+S: Save nav graph".to_string(),
                "Ctrl+L: Load nav graph".to_string(),
                "".to_string(),
            ],
        });

        sections.push(HelpSection {
            title: "AMBIENT LIGHT".to_string(),
            lines: vec![
                "+/-: Adjust ambient light (when no object selected)".to_string(),
                "".to_string(),
            ],
        });

        sections
    }
}

struct HelpSection {
    title: String,
    lines: Vec<String>,
}
