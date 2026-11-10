/* window.rs
 *
 * Copyright 2026 blushcream
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * 	http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use gtk::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use std::cell::Cell;
use std::rc::Rc;
use std::cell::RefCell;

use rodio::{OutputStreamBuilder, Sink};
use rodio::source::{SineWave, Source};
use std::time::Duration;

use gettextrs::gettext;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "src/window.blp")]
    pub struct MorsyWindow {
        // Template widgets
        #[template_child]
        pub input_text: TemplateChild<gtk::TextView>,
        #[template_child]
        pub input_morse: TemplateChild<gtk::TextView>,
        #[template_child]
        pub play_button: TemplateChild<gtk::Button>,

        pub audio_stream: RefCell<Option<rodio::OutputStream>>,
        pub audio_sink: RefCell<Option<rodio::Sink>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MorsyWindow {
        const NAME: &'static str = "MorsyWindow";
        type Type = super::MorsyWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MorsyWindow {

        fn constructed(&self) {
            self.parent_constructed();

            let updating = Rc::new(Cell::new(false));

            let input_text = self.input_text.clone();
            let input_morse = self.input_morse.clone();
            let window = self.obj().clone();

            let updating_morse = updating.clone();
            let updating_text = updating.clone();

            let obj = self.obj().clone();
            let play_button = self.play_button.clone();
            let button = play_button.clone();

            let audio_morse_textbox = input_morse.clone();

            // Connect the play button

            play_button.clone().connect_clicked(move |_| {

                let imp = obj.imp();

                if let Some(sink) = imp.audio_sink.borrow().as_ref() {
                    if !sink.empty() {

                        glib::g_message!("Morsy", "Stopping audio...");

                        sink.stop();

                        button.remove_css_class("destructive-action");
                        button.set_icon_name("media-playback-start-symbolic");
                        button.set_tooltip_text(Some(&gettext("Play")));

                        return;
                    }
                }

                glib::g_message!("Morsy", "Playing audio...");

                let mut stream = match OutputStreamBuilder::open_default_stream() {
                    Ok(stream) => stream,
                    Err(error) => {
                        glib::g_warning!("Morsy", "Failed to open audio output: {error}");
                        return;
                    }
                };

                stream.log_on_drop(false);

                let sink = Sink::connect_new(stream.mixer());

                let morse_code = audio_morse_textbox.buffer().text(
                    &audio_morse_textbox.buffer().start_iter(),
                    &audio_morse_textbox.buffer().end_iter(),
                    false,
                );

                if morse_code == "" {
                    return;
                }

                button.add_css_class("destructive-action");
                button.set_icon_name("media-playback-pause-symbolic");
                button.set_tooltip_text(Some(&gettext("Stop")));

                play_morse(&sink, &morse_code);

                obj.imp().audio_stream.replace(Some(stream));
                obj.imp().audio_sink.replace(Some(sink));

                let obj_timeout = obj.clone();
                let button_timeout = play_button.clone();

                // Restore the normal button behaviour after the play has completed

                glib::timeout_add_local(Duration::from_millis(100), move || {
                    if let Some(sink) = obj_timeout.imp().audio_sink.borrow().as_ref() {
                        if sink.empty() {
                            button_timeout.remove_css_class("destructive-action");
                            button_timeout.set_icon_name("media-playback-start-symbolic");
                            button_timeout.set_tooltip_text(Some(&gettext("Play")));
                            return glib::ControlFlow::Break;
                        }
                    }

                    glib::ControlFlow::Continue
                });
            });

            // Connect the input box changes (text one)

            input_text.buffer().connect_changed(move |buffer| {

                if updating_morse.get() {
                    return;
                }

                updating_morse.set(true);

                let morse = input_text.buffer().text(
                    &buffer.start_iter(),
                    &buffer.end_iter(),
                    false
                );

                let mut text = String::new();

                for c in morse.chars() {
                    text.push_str(window.to_morse(c));
                    text.push_str(" ");

                }

                input_morse.buffer().set_text(&text);

                updating_morse.set(false);
            });

            // More of cloned variables

            let input_text = self.input_text.clone();
            let input_morse = self.input_morse.clone();
            let window = self.obj().clone();

            // Connect the input box changes (morse one)

            input_morse.buffer().connect_changed(move |buffer| {

                if updating_text.get() {
                    return;
                }

                updating_text.set(true);

                let morse = input_morse.buffer().text(
                    &buffer.start_iter(),
                    &buffer.end_iter(),
                    false
                );

                let mut text = String::new();

                for code in morse.split_whitespace() {
                    text.push(window.to_text(code));
                }

                input_text.buffer().set_text(&text);

                updating_text.set(false);
            });

            glib::g_message!("Morsy", "App has been launched");

            self.input_text.buffer().set_text("WRITE MORSE HERE");
        }
    }
    impl WidgetImpl for MorsyWindow {}
    impl WindowImpl for MorsyWindow {}
    impl ApplicationWindowImpl for MorsyWindow {}
    impl AdwApplicationWindowImpl for MorsyWindow {}
}

glib::wrapper! {
    pub struct MorsyWindow(ObjectSubclass<imp::MorsyWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,        @implements gio::ActionGroup, gio::ActionMap;
}

impl MorsyWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    // Morse backend

    fn to_morse(&self, c: char) -> &'static str {
        match c.to_ascii_uppercase() {
            'A' => ".-",
            'B' => "-...",
            'C' => "-.-.",
            'D' => "-..",
            'E' => ".",
            'F' => "..-.",
            'G' => "--.",
            'H' => "....",
            'I' => "..",
            'J' => ".---",
            'K' => "-.-",
            'L' => ".-..",
            'M' => "--",
            'N' => "-.",
            'O' => "---",
            'P' => ".--.",
            'Q' => "--.-",
            'R' => ".-.",
            'S' => "...",
            'T' => "-",
            'U' => "..-",
            'V' => "...-",
            'W' => ".--",
            'X' => "-..-",
            'Y' => "-.--",
            'Z' => "--..",
            '.' => ".-.-.-",
            ',' => "--..--",
            '?' => "..--..",
            '\'' => ".----.",
            '!' => "-.-.--",
            '/' => "-..-.",
            '(' => "-.--.",
            ')' => "-.--.-",
            '&' => ".-...",
            ':' => "---...",
            ';' => "-.-.-.",
            '=' => "-...-",
            '+' => ".-.-.",
            '-' => "-....-",
            '_' => "..--.-",
            '"' => ".-..-.",
            '$' => "...-..-",
            '@' => ".--.-.",
            ' ' => "/",
            '0' => "-----",
            '1' => ".----",
            '2' => "..---",
            '3' => "...--",
            '4' => "....-",
            '5' => ".....",
            '6' => "-....",
            '7' => "--...",
            '8' => "---..",
            '9' => "----.",
            _ => "",
        }
    }

    fn to_text(&self, morse: &str) -> char {
        match morse {
            ".-" => 'A',
            "-..." => 'B',
            "-.-." => 'C',
            "-.." => 'D',
            "." => 'E',
            "..-." => 'F',
            "--." => 'G',
            "...." => 'H',
            ".." => 'I',
            ".---" => 'J',
            "-.-" => 'K',
            ".-.." => 'L',
            "--" => 'M',
            "-." => 'N',
            "---" => 'O',
            ".--." => 'P',
            "--.-" => 'Q',
            ".-." => 'R',
            "..." => 'S',
            "-" => 'T',
            "..-" => 'U',
            "...-" => 'V',
            ".--" => 'W',
            "-..-" => 'X',
            "-.--" => 'Y',
            "--.." => 'Z',
            ".-.-.-" => '.',
            "--..--" => ',',
            "..--.." => '?',
            ".----." => '\'',
            "-.-.--" => '!',
            "-..-." => '/',
            "-.--." => '(',
            "-.--.-" => ')',
            ".-..." => '&',
            "---..." => ':',
            "-.-.-." => ';',
            "-...-" => '=',
            ".-.-." => '+',
            "-....-" => '-',
            "..--.-" => '_',
            ".-..-." => '"',
            "...-..-" => '$',
            ".--.-." => '@',
            "/" => ' ',
            // Added Numbers
            "-----" => '0',
            ".----" => '1',
            "..---" => '2',
            "...--" => '3',
            "....-" => '4',
            "....." => '5',
            "-...." => '6',
            "--..." => '7',
            "---.." => '8',
            "----." => '9',
            _ => ' ',
        }
    }
}

// Audio backend

fn play_morse(sink: &Sink, morse: &str) {

    // Speed
    const UNIT: u64 = 80;

    let codes: Vec<&str> = morse.split_whitespace().collect();

    for (index, code) in codes.iter().enumerate() {
        if *code == "/" {
            sink.append(
                SineWave::new(600.0)
                    .take_duration(Duration::from_millis(UNIT * 7))
                    .amplify(0.0),
            );
            continue;
        }

        for (symbol_index, symbol) in code.chars().enumerate() {
            let duration = match symbol {
                '.' => UNIT,
                '-' => UNIT * 3,
                _ => continue,
            };

            sink.append(
                SineWave::new(600.0)
                    .take_duration(Duration::from_millis(duration))
                    .amplify(0.2),
            );

            if symbol_index + 1 < code.len() {
                sink.append(
                    SineWave::new(600.0)
                        .take_duration(Duration::from_millis(UNIT))
                        .amplify(0.0),
                );
            }
        }

        if let Some(next) = codes.get(index + 1) {
            if *next != "/" {
                sink.append(
                    SineWave::new(600.0)
                        .take_duration(Duration::from_millis(UNIT * 3))
                        .amplify(0.0),
                );
            }
        }
    }
}
