use eframe::egui::{self, Checkbox, TextEdit};
use egui_extras::{Size, TableBuilder};
use pam_u2f_mapping::{Mapping, MappingFile};

#[derive(Clone, Debug, Default)]
pub struct Editor {
	/// The mapping file we're editing
	mapping: Option<MappingView>,
	/// The file path to open
	file: String,
	/// Error message
	error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct MappingView {
	/// The mapping data
	mapping: MappingFile,
	/// Selected user
	selected: Option<usize>,
	/// Add user text entry
	new_user: String,
}

impl MappingView {
	/// Gets a reference to the selected [`Mapping`]
	fn selected(&self) -> Option<&Mapping> {
		match self.selected {
			Some(selected) => self.mapping.mappings.get(selected),
			None => None,
		}
	}
}

#[derive(Debug, Clone, Copy, Hash)]
enum Id {
	LeftPanel,
}

impl Editor {
	pub fn new() -> Self {
		Self::default()
	}
}

impl eframe::App for Editor {
	fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
		if let Some(view) = &mut self.mapping {
			egui::SidePanel::left(Id::LeftPanel).show(ctx, |ui| {
				ui.heading("Users");
				for (idx, entry) in view.mapping.mappings.iter().enumerate() {
					ui.selectable_value(&mut view.selected, Some(idx), &entry.user);
				}
				ui.horizontal(|ui| {
					ui.add(TextEdit::singleline(&mut view.new_user).desired_width(100.0));
					if ui.button("+").clicked() {
						view.mapping.mappings.push(Mapping {
							user: view.new_user.drain(..).collect(),
							keys: Vec::new(),
						})
					}
				});
			});
			egui::CentralPanel::default().show(ctx, |ui| {
				let selected = match view.selected() {
					None => {
						ui.label("No user selected");
						return;
					}
					Some(selected) => selected,
				};
				ui.vertical(|ui| {
					TableBuilder::new(ui)
						.resizable(true)
						.striped(true)
						// type
						.column(Size::initial(50.0).at_least(50.0).at_most(200.0))
						// Handle
						.column(Size::initial(350.0))
						// pin
						.column(Size::initial(50.0))
						// presence
						.column(Size::initial(100.0))
						.scroll(false)
						.header(20.0, |mut header| {
							header.col(|ui| {
								ui.heading("Type")
									.on_hover_text("The algorithm used for asymmetric encryption");
							});
							header.col(|ui| {
								ui.heading("Handle")
									.on_hover_text("The identifier for the key");
							});
							header.col(|ui| {
								ui.heading("PIN").on_hover_text("Requires PIN entry");
							});
							header.col(|ui| {
								ui.heading("Presence")
									.on_hover_text("Requires presence check");
							});
						})
						.body(|mut body| {
							for key in &selected.keys {
								body.row(16.0, |mut row| {
									// type
									row.col(|ui| drop(ui.label(&key.kind)));
									// Handle
									row.col(|ui| drop(ui.label(&key.handle)));
									// pin
									row.col(|ui| {
										let mut pin = key.flags.contains(&String::from("pin"));
										ui.add_enabled(false, Checkbox::new(&mut pin, ""));
									});
									// presence
									row.col(|ui| {
										let mut presence =
											key.flags.contains(&String::from("presence"));
										ui.add_enabled(false, Checkbox::new(&mut presence, ""));
									});
								})
							}
						});
				});
				if ui.button("Register key").clicked() {}
			});
		} else {
			// Show file picker
			egui::CentralPanel::default().show(ctx, |ui| {
				if self.file.is_empty() {
					self.file = String::from("/home/amanda/u2f_mappings");
				}
				ui.vertical_centered(|ui| {
					ui.add_space(ui.available_height() / 2.1);
					ui.group(|ui| {
						ui.set_max_width(300.0);
						ui.text_edit_singleline(&mut self.file);
						if ui.button("Open").clicked() {
							tracing::info!("Opening {}", &self.file);
							let result: Result<MappingFile, Box<dyn std::error::Error>> = (|| {
								let data: MappingFile =
									std::fs::read_to_string(&self.file)?.parse()?;
								Ok(data)
							})();
							match result {
								Ok(mapping) => {
									self.mapping = Some(MappingView {
										mapping,
										selected: None,
										new_user: String::new(),
									});
								}
								Err(err) => {
									self.error = Some(err.to_string());
								}
							}
						}
						if let Some(error) = self.error.as_deref() {
							ui.label(error);
						}
					})
				});
			});
		}
	}
}
