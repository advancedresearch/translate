use eframe::egui;

fn main() -> eframe::Result {
    let file = std::env::args_os()
        .nth(1)
        .and_then(|s| s.into_string().ok());
    if let Some(file) = file {
        let data = translate::load(&file)
            .unwrap_or_else(|_| translate::new());

        // env_logger::init();
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([500.0, 800.0]),
            ..Default::default()
        };
        eframe::run_native(
            &format!("Translate v0.1 - {}", file),
            options,
            Box::new(|cc| {
                // At this point Retine display is not detected.
                // Make text larger for ease of use.
                cc.egui_ctx.set_pixels_per_point(2.0);

                // This gives us image support:
                // egui_extras::install_image_loaders(&cc.egui_ctx);

                Ok(Box::new(TranslateApp::new(file, data)))
            }),
        )
    } else {
        eprintln!("translate <file.tr>");
        return Ok(());
    }
}

pub struct TranslateApp {
    pub ind: usize,
    pub data: translate::Data,
    pub file: String,
    pub lookup: String,
    pub answer: String,
    pub answer_id: Option<usize>,
}

impl TranslateApp {
    pub fn new(file: String, data: Vec<(String, String)>) -> TranslateApp {
        TranslateApp {
            ind: 0,
            data,
            file,
            lookup: "".into(),
            answer: "".into(),
            answer_id: None,
        }
    }
}

impl eframe::App for TranslateApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame
    ) {
        // This is higher here due to Retina display.
        // ctx.set_pixels_per_point(4.0);

        egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let ind = self.ind;

            ui.horizontal(|ui| {
                if ui.button("||<").clicked() {
                    use translate::Token;

                    for i in (0..ind).rev() {
                        if Token::tokenize(&self.data[i].0).len() > 1 {
                            self.ind = i;
                            break;
                        }
                    }
                }

                if ui.button(">||").clicked() {
                    use translate::Token;

                    for i in (ind + 1)..self.data.len() {
                        if Token::tokenize(&self.data[i].0).len() > 1 {
                            self.ind = i;
                            break;
                        }
                    }
                }

                if ui.button("<<").clicked() {
                    if ind >= 100 {self.ind -= 100}
                }
                if ui.button(">>").clicked() {
                    if ind + 100 < self.data.len() {self.ind += 100}
                }

                ui.label(format!("{} of {}", ind + 1, self.data.len()));

                if ui.button(">*").clicked() {
                    self.data.insert(ind + 1, ("".into(), "".into()));
                    self.ind = ind + 1;
                }

                if ui.button("x").clicked() {
                    if self.data.len() > 0 {
                        self.data.remove(ind);
                        self.ind = ind.min(self.data.len() - 1);
                    }
                }
            });

            ui.horizontal(|ui| {
                if ui.button("*").clicked() {
                    self.data.push(("".into(), "".into()));
                    self.ind = self.data.len() - 1;
                }

                if ui.button("|<").clicked() {
                    self.ind = 0;
                }

                if ui.button("<").clicked() {
                    self.ind = if ind > 0 {self.ind - 1} else {0};
                }

                if ui.button(">").clicked() {
                    self.ind = if ind + 2 <= self.data.len() {
                            self.ind + 1
                        } else {
                            self.data.len() - 1
                        };
                }

                if ui.button(">|").clicked() {
                    self.ind = self.data.len() - 1;
                }

                if ui.button("<>").clicked() {
                    if ind > 0 {
                        let tmp = self.data[ind].clone();
                        self.data[ind] = self.data[ind - 1].clone();
                        self.data[ind - 1] = tmp;
                    }
                }

                if ui.button("translate").clicked() {
                    use translate::Token;

                    let tokens = Token::tokenize(&self.data[ind].0);
                    let mut res = String::new();
                    for token in tokens.into_iter() {
                        match token {
                            Token::Separator(ch) => res.push(ch),
                            Token::Word(word) => {
                                let mut found = false;
                                for d in &self.data {
                                    if d.0 == word {
                                        res.push('[');
                                        res.push_str(&d.1);
                                        res.push(']');
                                        found = true;
                                        break;
                                    }
                                }

                                if !found {
                                    res.push_str(&word);
                                }
                            }
                        }
                    }

                    self.data[ind].1 = res;
                }

                if ui.button("+").clicked() {
                    use translate::Token;

                    let tokens = Token::tokenize(&self.data[ind].0);
                    let mut i = 0;
                    for token in tokens.into_iter() {
                        match token {
                            Token::Separator(_) => {}
                            Token::Word(word) => {
                                let mut found = false;
                                for d in &self.data {
                                    if d.0 == word {
                                        found = true;
                                        break;
                                    }
                                }

                                if !found {
                                    i += 1;
                                    self.data.insert(ind + i, (word, "".into()));
                                }
                            }
                        }
                    }
                }
            });

            let ind = self.ind;

            ui.text_edit_multiline(&mut self.data[ind].0);
            ui.text_edit_multiline(&mut self.data[ind].1);

            ui.text_edit_singleline(&mut self.lookup);
            if let Some(id) = self.answer_id {
                if ui.label(format!("{} (page {})",
                    self.answer.clone(), id + 1)).clicked()
                {
                    ui.output_mut(|o| o.copied_text = self.answer.clone());
                }
            } else {
                ui.label("");
            }

            if ui.button("Search").clicked() {
                let mut found = false;
                for (i, d) in self.data.iter().enumerate() {
                    if d.0 == self.lookup {
                        self.answer = d.1.clone();
                        self.answer_id = Some(i);
                        found = true;
                        break;
                    }
                }

                if !found {self.answer = "(not found)".into()}
            }

            if ui.button("Save").clicked() {
                if let Err(err) = translate::save(&self.file, &self.data) {
                    eprintln!("ERROR:\nCould not write to file `{}`\n{}", self.file, err);
                }
            }
        })});
    }
}
