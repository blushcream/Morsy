    /* application.rs
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

use gettextrs::gettext;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::config::VERSION;
use crate::MorsyWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct MorsyApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for MorsyApplication {
        const NAME: &'static str = "MorsyApplication";
        type Type = super::MorsyApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for MorsyApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<control>q"]);
        }
    }

    impl ApplicationImpl for MorsyApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let application = self.obj();
            // Get the current window or create one if necessary
            let window = application.active_window().unwrap_or_else(|| {
                let window = MorsyWindow::new(&*application);
                window.upcast()
            });

            // Ask the window manager/compositor to present the window
            window.present();
        }
    }

    impl GtkApplicationImpl for MorsyApplication {}
    impl AdwApplicationImpl for MorsyApplication {}
}

glib::wrapper! {
    pub struct MorsyApplication(ObjectSubclass<imp::MorsyApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl MorsyApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .property("resource-base-path", "/org/blushCream/Morsy")
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        self.add_action_entries([quit_action, about_action]);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutDialog::builder()
            .application_name("Morsy")
            .application_icon("io.github.blushcream.Morsy")
            .developer_name("blushcream")
            .version(VERSION)
            .developers(vec!["blushcream"])
            // Translators: Replace "translator-credits" with your name/username, and optionally an email or URL.
            .translator_credits(&gettext("translator-credits"))
            .copyright("© 2026 blushcream")
            .build();

        about.set_license_type(gtk::License::Apache20);
        about.set_artists(&["blushcream, Morsy App Icon"]);
        about.set_comments("A simple, beautiful Morse Code translator with audio playback. Made in Rust with GTK4 and Libadwaita.");

        about.present(Some(&window));
    }
}
