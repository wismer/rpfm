// In this file we create the UI of the RPFM, and control it (events, updates, etc...).

#![windows_subsystem = "windows"]

extern crate gtk;
extern crate gdk;
extern crate num;

use std::path::PathBuf;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    AboutDialog, Builder, MenuItem, Window, WindowPosition, FileChooserDialog,
    TreeView, TreeSelection, TreeStore, MessageDialog, ScrolledWindow,
    CellRendererText, TreeViewColumn, Popover, Entry, CheckMenuItem, Button
};

mod common;
mod ui;
mod pack_file_manager;

// This macro is used to clone the variables into the closures without the compiler protesting.
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

// One Function to rule them all, One Function to find them,
// One Function to bring them all and in the darkness bind them.
fn main() {

    // Init GTK3. Boilerplate code.
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    // We import the Glade design and get all the UI objects into variables.
    let glade_design = include_str!("glade/main.glade");
    let builder = Builder::new_from_string(glade_design);

    let window: Window = builder.get_object("gtk_window").expect("Couldn't get gtk_window");

    let packed_file_data_display: ScrolledWindow = builder.get_object("gtk_packed_file_data_display").expect("Couldn't get gtk_packed_file_data_display");

    let window_about: AboutDialog = builder.get_object("gtk_window_about").expect("Couldn't get gtk_window_about");
    let error_dialog: MessageDialog = builder.get_object("gtk_error_dialog").expect("Couldn't get gtk_error_dialog");
    let success_dialog: MessageDialog = builder.get_object("gtk_success_dialog").expect("Couldn't get gtk_success_dialog");
    let rename_popover: Popover = builder.get_object("gtk_rename_popover").expect("Couldn't get gtk_rename_popover");

    let rename_popover_text_entry: Entry = builder.get_object("gtk_rename_popover_text_entry").expect("Couldn't get gtk_rename_popover_text_entry");

    let file_chooser_open_packfile_dialog: FileChooserDialog = builder.get_object("gtk_file_chooser_open_packfile").expect("Couldn't get gtk_file_chooser_open_packfile");
    let file_chooser_save_packfile_dialog: FileChooserDialog = builder.get_object("gtk_file_chooser_save_packfile").expect("Couldn't get gtk_file_chooser_save_packfile");
    let file_chooser_add_file_to_packfile: FileChooserDialog = builder.get_object("gtk_file_chooser_add_file_to_packfile").expect("Couldn't get gtk_file_chooser_add_file_to_packfile");
    let file_chooser_add_folder_to_packfile: FileChooserDialog = builder.get_object("gtk_file_chooser_add_folder_to_packfile").expect("Couldn't get gtk_file_chooser_add_folder_to_packfile");
    let file_chooser_extract_file: FileChooserDialog = builder.get_object("gtk_file_chooser_extract_file").expect("Couldn't get gtk_file_chooser_extract_file");
    let file_chooser_extract_folder: FileChooserDialog = builder.get_object("gtk_file_chooser_extract_folder").expect("Couldn't get gtk_file_chooser_extract_folder");

    let tree_view_add_file: Button = builder.get_object("gtk_context_menu_tree_view_add_file").expect("Couldn't get gtk_context_menu_tree_view_add_file");
    let tree_view_add_folder: Button = builder.get_object("gtk_context_menu_tree_view_add_folder").expect("Couldn't get gtk_context_menu_tree_view_add_folder");
    let tree_view_delete_file: Button = builder.get_object("gtk_context_menu_tree_view_delete_file").expect("Couldn't get gtk_context_menu_tree_view_delete_file");
    let tree_view_extract_file: Button = builder.get_object("gtk_context_menu_tree_view_extract_file").expect("Couldn't get gtk_context_menu_tree_view_extract_file");

    let top_menu_file: MenuItem = builder.get_object("gtk_top_menu_file").expect("Couldn't get gtk_top_menu_file");
    let top_menu_special_stuff: MenuItem = builder.get_object("gtk_top_menu_special_stuff").expect("Couldn't get gtk_top_menu_special_stuff");

    let context_menu_tree_view: Popover = builder.get_object("gtk_context_menu_tree_view").expect("Couldn't get gtk_context_menu_tree_view");

    let top_menu_file_new_packfile: MenuItem = builder.get_object("gtk_top_menu_file_new_packfile").expect("Couldn't get gtk_top_menu_file_new_packfile");
    let top_menu_file_open_packfile: MenuItem = builder.get_object("gtk_top_menu_file_open_packfile").expect("Couldn't get gtk_top_menu_file_open_packfile");
    let top_menu_file_save_packfile: MenuItem = builder.get_object("gtk_top_menu_file_save_packfile").expect("Couldn't get gtk_top_menu_file_save_packfile");
    let top_menu_file_save_packfile_as: MenuItem = builder.get_object("gtk_top_menu_file_save_packfile_as").expect("Couldn't get gtk_top_menu_file_save_packfile_as");
    let top_menu_file_quit: MenuItem = builder.get_object("gtk_top_menu_file_quit").expect("Couldn't get gtk_top_menu_file_quit");
    let top_menu_special_patch_ai: MenuItem = builder.get_object("gtk_top_menu_special_patch_ai").expect("Couldn't get gtk_top_menu_special_patch_ai");
    let top_menu_about_about: MenuItem = builder.get_object("gtk_top_menu_about_about").expect("Couldn't get gtk_top_menu_about_about");

    let top_menu_file_change_packfile_type: MenuItem = builder.get_object("gtk_top_menu_file_select_packfile_type").expect("Couldn't get gtk_top_menu_file_select_packfile_type");
    let top_menu_file_change_packfile_type_boot: CheckMenuItem = builder.get_object("gtk_top_menu_file_select_packfile_type1").expect("Couldn't get gtk_top_menu_file_select_packfile_type1");
    let top_menu_file_change_packfile_type_release: CheckMenuItem = builder.get_object("gtk_top_menu_file_select_packfile_type2").expect("Couldn't get gtk_top_menu_file_select_packfile_type2");
    let top_menu_file_change_packfile_type_patch: CheckMenuItem = builder.get_object("gtk_top_menu_file_select_packfile_type3").expect("Couldn't get gtk_top_menu_file_select_packfile_type3");
    let top_menu_file_change_packfile_type_mod: CheckMenuItem = builder.get_object("gtk_top_menu_file_select_packfile_type4").expect("Couldn't get gtk_top_menu_file_select_packfile_type4");
    let top_menu_file_change_packfile_type_movie: CheckMenuItem = builder.get_object("gtk_top_menu_file_select_packfile_type5").expect("Couldn't get gtk_top_menu_file_select_packfile_type5");

    let folder_tree_view: TreeView = builder.get_object("gtk_folder_tree_view").expect("Couldn't get gtk_folder_tree_view");
    let folder_tree_selection: TreeSelection = builder.get_object("gtk_folder_tree_view_selection").expect("Couldn't get gtk_folder_tree_view_selection");

    // The TreeView's stuff is created manually here, as I had problems creating it in Glade.
    let folder_tree_store = TreeStore::new(&[String::static_type()]);
    folder_tree_view.set_model(Some(&folder_tree_store));

    let column = TreeViewColumn::new();
    let cell = CellRendererText::new();
    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", 0);

    folder_tree_view.append_column(&column);
    folder_tree_view.set_enable_tree_lines(true);
    folder_tree_view.set_enable_search(false);
    folder_tree_view.set_rules_hint(true);
    window.set_position(WindowPosition::Center);

    // We bring up the main window.
    window.show_all();

    // We also create a dummy PackFile we're going to use to store all the data from the opened Packfile.
    let pack_file_decoded = Rc::new(RefCell::new(pack_file_manager::pack_file::PackFile::new()));

    // End of the "Getting Ready" part.
    // From here, it's all event handling.

    // First, we catch the close window event, and close the program when we do it.
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    /*
    --------------------------------------------------------
                     Superior Menu: "File"
    --------------------------------------------------------
    */

    // When we open the menu, we check if we need to enable or disable his buttons first.
    top_menu_file.connect_activate(clone!(
        top_menu_file_save_packfile,
        top_menu_file_save_packfile_as,
        top_menu_file_change_packfile_type,
        pack_file_decoded => move |_| {

        // If the current PackFile has no name, we haven't open or created one, so disable all the
        // options that need a PackFile opened. Otherwise enable them.
        if pack_file_decoded.borrow().pack_file_extra_data.file_name.is_empty() {
            top_menu_file_save_packfile.set_sensitive(false);
            top_menu_file_save_packfile_as.set_sensitive(false);
            top_menu_file_change_packfile_type.set_sensitive(false);
        }
        else {
            top_menu_file_save_packfile.set_sensitive(true);
            top_menu_file_save_packfile_as.set_sensitive(true);
            top_menu_file_change_packfile_type.set_sensitive(true);
        }
    }));


    // When we hit the "New PackFile" button.
    top_menu_file_new_packfile.connect_activate(clone!(
        pack_file_decoded,
        folder_tree_store,
        top_menu_file_change_packfile_type_mod => move |_| {

        // We just create a new PackFile with a name, set his type to Mod and update the
        // TreeView to show it.
        *pack_file_decoded.borrow_mut() = pack_file_manager::new_packfile("Unkown.pack".to_string());
        ui::update_tree_view(&folder_tree_store, &*pack_file_decoded.borrow());
        top_menu_file_change_packfile_type_mod.set_active(true);
    }));


    // When we hit the "Open PackFile" button.
    top_menu_file_open_packfile.connect_activate(clone!(
        error_dialog,
        pack_file_decoded,
        folder_tree_store,
        top_menu_file_change_packfile_type_boot,
        top_menu_file_change_packfile_type_release,
        top_menu_file_change_packfile_type_patch,
        top_menu_file_change_packfile_type_mod,
        top_menu_file_change_packfile_type_movie => move |_| {

        // When we select the file to open, we get his path, open it and, if there has been no
        // errors, decode it, update the TreeView to show it and check his type for the Change FilePack
        // Type option in the File menu.
        if file_chooser_open_packfile_dialog.run() == gtk::ResponseType::Ok.into() {
            let pack_file_path = file_chooser_open_packfile_dialog.get_filename().expect("Couldn't open file");
            match pack_file_manager::open_packfile(pack_file_path) {
                Ok(pack_file_opened) => {

                    *pack_file_decoded.borrow_mut() = pack_file_opened;
                    ui::update_tree_view(&folder_tree_store, &*pack_file_decoded.borrow());

                    // We choose the right option, depending on our PackFile.
                    if pack_file_decoded.borrow().pack_file_header.pack_file_type == 0u32 {
                        top_menu_file_change_packfile_type_boot.set_active(true);
                    }
                    else if pack_file_decoded.borrow().pack_file_header.pack_file_type == 1u32{
                        top_menu_file_change_packfile_type_release.set_active(true);
                    }
                    else if pack_file_decoded.borrow().pack_file_header.pack_file_type == 2u32{
                        top_menu_file_change_packfile_type_patch.set_active(true);
                    }
                    else if pack_file_decoded.borrow().pack_file_header.pack_file_type == 3u32{
                        top_menu_file_change_packfile_type_mod.set_active(true);
                    }
                    else if pack_file_decoded.borrow().pack_file_header.pack_file_type == 4u32{
                        top_menu_file_change_packfile_type_movie.set_active(true);
                    }
                }
                Err(e) => {
                    ui::show_dialog(&error_dialog, e);
                }
            }
        }
        file_chooser_open_packfile_dialog.hide_on_delete();
    }));


    // When we hit the "Save PackFile" button
    top_menu_file_save_packfile.connect_activate(clone!(
        success_dialog,
        error_dialog,
        pack_file_decoded,
        folder_tree_view,
        folder_tree_store,
        folder_tree_selection,
        file_chooser_save_packfile_dialog => move |_| {

        // First, we check if our PackFile has a path. If it doesn't have it, we launch the Save
        // Dialog and set the current name in the extry of the dialog to his name.
        // When we hit "Accept", we get the selected path, encode the PackFile, and save it to that
        // path. After that, we update the TreeView to reflect the name change and hide the dialog.
        let mut pack_file_path: Option<PathBuf> = None;
        if pack_file_decoded.borrow().pack_file_extra_data.file_path.is_empty() {
            file_chooser_save_packfile_dialog.set_current_name(&pack_file_decoded.borrow().pack_file_extra_data.file_name);
            if file_chooser_save_packfile_dialog.run() == gtk::ResponseType::Ok.into() {
                pack_file_path = Some(file_chooser_save_packfile_dialog.get_filename().expect("Couldn't open file"));
                match pack_file_manager::save_packfile( &mut *pack_file_decoded.borrow_mut(), pack_file_path) {
                    Ok(result) => {
                        ui::show_dialog(&success_dialog, result)
                    }
                    Err(result) => {
                        ui::show_dialog(&error_dialog, result)
                    }
                }

                ui::update_tree_view_expand_path(
                    &folder_tree_store,
                    &*pack_file_decoded.borrow(),
                    &folder_tree_selection,
                    &folder_tree_view,
                    false
                );
            }
            file_chooser_save_packfile_dialog.hide_on_delete();
        }

        // If the PackFile has a path, we just encode it and save it into that path.
        else {
            match pack_file_manager::save_packfile( &mut *pack_file_decoded.borrow_mut(), pack_file_path) {
                Ok(result) => {
                    ui::show_dialog(&success_dialog, result)
                }
                Err(result) => {
                    ui::show_dialog(&error_dialog, result)
                }
            }
        }
    }));


    // When we hit the "Save PackFile as" button.
    top_menu_file_save_packfile_as.connect_activate(clone!(
        success_dialog,
        error_dialog,
        pack_file_decoded,
        folder_tree_view,
        folder_tree_store,
        folder_tree_selection,
        file_chooser_save_packfile_dialog => move |_| {

        // We first set the current file of the Save dialog to the PackFile's name. Then we just
        // encode it and save it in the path selected. After that, we update the TreeView to reflect
        // the name change and hide the dialog.
        file_chooser_save_packfile_dialog.set_current_name(&pack_file_decoded.borrow().pack_file_extra_data.file_name);
        if file_chooser_save_packfile_dialog.run() == gtk::ResponseType::Ok.into() {
            match pack_file_manager::save_packfile(
                &mut *pack_file_decoded.borrow_mut(),
               Some(file_chooser_save_packfile_dialog.get_filename().expect("Couldn't open file"))) {
                Ok(result) => {
                    ui::show_dialog(&success_dialog, result);
                }
                Err(result) => {
                    ui::show_dialog(&error_dialog, result)
                }
            }

            ui::update_tree_view_expand_path(
                &folder_tree_store,
                &*pack_file_decoded.borrow(),
                &folder_tree_selection,
                &folder_tree_view,
                false
            );
        }
        file_chooser_save_packfile_dialog.hide_on_delete();
    }));


    // When changing the type of the PackFile... we just change his pack_file_type variable. Nothing complex.
    top_menu_file_change_packfile_type_boot.connect_toggled(clone!(
        top_menu_file_change_packfile_type_boot,
        pack_file_decoded => move |_| {
        if top_menu_file_change_packfile_type_boot.get_active() {
            pack_file_decoded.borrow_mut().pack_file_header.pack_file_type = 0;
        }
    }));
    top_menu_file_change_packfile_type_release.connect_toggled(clone!(
        top_menu_file_change_packfile_type_release,
        pack_file_decoded => move |_| {
        if top_menu_file_change_packfile_type_release.get_active() {
            pack_file_decoded.borrow_mut().pack_file_header.pack_file_type = 1;
        }
    }));
    top_menu_file_change_packfile_type_patch.connect_toggled(clone!(
        top_menu_file_change_packfile_type_patch,
        pack_file_decoded => move |_| {
        if top_menu_file_change_packfile_type_patch.get_active() {
            pack_file_decoded.borrow_mut().pack_file_header.pack_file_type = 2;
        }
    }));
    top_menu_file_change_packfile_type_mod.connect_toggled(clone!(
        top_menu_file_change_packfile_type_mod,
        pack_file_decoded => move |_| {
        if top_menu_file_change_packfile_type_mod.get_active() {
            pack_file_decoded.borrow_mut().pack_file_header.pack_file_type = 3;
        }
    }));
    top_menu_file_change_packfile_type_movie.connect_toggled(clone!(
        top_menu_file_change_packfile_type_movie,
        pack_file_decoded => move |_| {
        if top_menu_file_change_packfile_type_movie.get_active() {
            pack_file_decoded.borrow_mut().pack_file_header.pack_file_type = 4;
        }
    }));


    // When we hit the "Quit" button.
    top_menu_file_quit.connect_activate(|_| {
        gtk::main_quit();
    });

    /*
    --------------------------------------------------------
                 Superior Menu: "Special Stuff"
    --------------------------------------------------------
    */

    // When we open the menu, we check if we need to enable or disable his buttons first.
    top_menu_special_stuff.connect_activate(clone!(
        top_menu_special_patch_ai,
        pack_file_decoded => move |_| {
        if pack_file_decoded.borrow().pack_file_extra_data.file_name.is_empty() {
            top_menu_special_patch_ai.set_sensitive(false);
        }
        else {
            top_menu_special_patch_ai.set_sensitive(true);
        }
    }));


    // When we hit the "Patch SiegeAI" button.
    top_menu_special_patch_ai.connect_activate(clone!(
    success_dialog,
    error_dialog,
    pack_file_decoded,
    folder_tree_view,
    folder_tree_store,
    folder_tree_selection => move |_| {

        // First, we try to patch the PackFile. If there are no errors, we save the result in a tuple.
        // Then we check that tuple and, if it's a success, we save the PackFile and update the TreeView.
        let mut sucessful_patching = (false, String::new());
        match pack_file_manager::patch_siege_ai(&mut *pack_file_decoded.borrow_mut()) {
            Ok(result) => {
                sucessful_patching = (true, result);
            }
            Err(result) => {
                ui::show_dialog(&error_dialog, result)
            }
        }
        if sucessful_patching.0 {
            match pack_file_manager::save_packfile( &mut *pack_file_decoded.borrow_mut(), None) {
                Ok(result) => {
                    ui::show_dialog(&success_dialog, format!("{}\n\n{}", sucessful_patching.1, result));
                }
                Err(_) => {
                    ui::show_dialog(&error_dialog, sucessful_patching.1)
                }
            }
            ui::update_tree_view_expand_path(
                &folder_tree_store,
                &*pack_file_decoded.borrow(),
                &folder_tree_selection,
                &folder_tree_view,
                false
            );
        }
    }));

    /*
    --------------------------------------------------------
                    Superior Menu: "About"
    --------------------------------------------------------
    */

    // When we hit the "About" button.
    top_menu_about_about.connect_activate(move |_| {
        window_about.run();
        window_about.hide_on_delete();
    });


    /*
    --------------------------------------------------------
                   Contextual TreeView Popup
    --------------------------------------------------------
    */

    // When we right-click the TreeView, we check if we need to enable or disable his buttons first.
    // Then we calculate the position where the popup must aim, and show it.
    //
    // NOTE: REMEMBER, WE OPEN THE POPUP HERE, BUT WE NEED TO CLOSED IT WHEN WE HIT HIS BUTTONS.
    folder_tree_view.connect_button_release_event(clone!(
        pack_file_decoded,
        folder_tree_view,
        folder_tree_selection,
        tree_view_add_file,
        tree_view_add_folder,
        tree_view_extract_file,
        context_menu_tree_view => move |_, button| {

        let button_val = button.get_button();
        if button_val == 3 && folder_tree_selection.count_selected_rows() > 0 {
            let tree_path = ui::get_tree_path_from_selection(&folder_tree_selection, false);
            for i in &*pack_file_decoded.borrow().pack_file_data.packed_files {
                // If the selected thing is a file
                if i.packed_file_path == tree_path {
                    tree_view_add_file.set_sensitive(false);
                    tree_view_add_folder.set_sensitive(false);
                    tree_view_extract_file.set_sensitive(true);
                    break;
                }
                else {
                    tree_view_add_file.set_sensitive(true);
                    tree_view_add_folder.set_sensitive(true);
                    tree_view_extract_file.set_sensitive(true);
                }
            }
            if tree_path.len() == 0 {
                tree_view_extract_file.set_sensitive(false);
            }
            let rect = ui::get_rect_for_popover(&folder_tree_selection, &folder_tree_view);

            context_menu_tree_view.set_pointing_to(&rect);
            context_menu_tree_view.popup();
        }
        Inhibit(false)
    }));


    // When we hit the "Add file" button.
    tree_view_add_file.connect_button_release_event(clone!(
        error_dialog,
        pack_file_decoded,
        folder_tree_view,
        folder_tree_store,
        folder_tree_selection,
        context_menu_tree_view => move |_,_| {

        // First, we hide the context menu, then we pick the file selected and add it to the Packfile.
        // After that, we update the TreeView.
        context_menu_tree_view.popdown();

        if file_chooser_add_file_to_packfile.run() == gtk::ResponseType::Ok.into() {
            let file_path = file_chooser_add_file_to_packfile.get_filename().expect("Couldn't open file");
            let tree_path = ui::get_tree_path_from_pathbuf(&file_path, &folder_tree_selection, true);
            let mut file_added = false;
            match pack_file_manager::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), file_path, tree_path) {
                Ok(_) => {
                    file_added = true;
                }
                Err(result) => {
                    ui::show_dialog(&error_dialog, result);
                }
            }
            if file_added {
                ui::update_tree_view_expand_path(
                    &folder_tree_store,
                    &*pack_file_decoded.borrow(),
                    &folder_tree_selection,
                    &folder_tree_view,
                    false
                );
            }
        }
        file_chooser_add_file_to_packfile.hide_on_delete();

        Inhibit(false)
    }));


    // When we hit the "Add folder" button.
    tree_view_add_folder.connect_button_release_event(clone!(
        error_dialog,
        pack_file_decoded,
        folder_tree_view,
        folder_tree_store,
        folder_tree_selection,
        context_menu_tree_view => move |_,_| {

        // First, we hide the context menu. Then we get the folder selected and we get all the files
        // in him and his subfolders. After that, for every one of those files, we strip his path,
        // leaving then with only the part that will be added to the PackedFile and we add it to the
        // PackFile. After all that, if we added any of the files to the PackFile, we update the
        // TreeView.
        context_menu_tree_view.popdown();
        if file_chooser_add_folder_to_packfile.run() == gtk::ResponseType::Ok.into() {
            let big_parent = file_chooser_add_folder_to_packfile.get_filename().unwrap();
            let mut big_parent_prefix = big_parent.clone();
            big_parent_prefix.pop();
            let file_path_list = ::common::get_files_from_subdir(&big_parent);
            let mut file_errors = 0;
            for i in file_path_list {
                match i.strip_prefix(&big_parent_prefix) {
                    Ok(filtered_path) => {
                        let tree_path = ui::get_tree_path_from_pathbuf(&filtered_path.to_path_buf(), &folder_tree_selection, false);
                        match pack_file_manager::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), i.to_path_buf(), tree_path) {
                            Ok(_) => {
                                // Do nothing, as we just want to know the errors.
                            }
                            Err(_) => {
                                file_errors += 1;
                            }
                        }
                    }
                    Err(_) => {
                        panic!("Error while trying to filter the path. This should never happend unless I break something while I'm getting the paths.");
                    }
                }
            }
            if file_errors > 0 {
                ui::show_dialog(&error_dialog, format!("{} file/s that you wanted to add already exist in the Packfile.", file_errors));
            }
            ui::update_tree_view_expand_path(
                &folder_tree_store,
                &*pack_file_decoded.borrow(),
                &folder_tree_selection,
                &folder_tree_view,
                false
            );
        }
        file_chooser_add_folder_to_packfile.hide_on_delete();

        Inhibit(false)
    }));


    // When we hit the "Delete file/folder" button.
    tree_view_delete_file.connect_button_release_event(clone!(
        pack_file_decoded,
        folder_tree_view,
        folder_tree_store,
        folder_tree_selection,
        context_menu_tree_view => move |_,_|{

        // We hide the context menu, then we get the selected file/folder, delete it and update the
        // TreeView. Pretty simple, actually.
        context_menu_tree_view.popdown();

        let tree_path = ui::get_tree_path_from_selection(&folder_tree_selection, false);
        pack_file_manager::delete_from_packfile(&mut *pack_file_decoded.borrow_mut(), tree_path);
        ui::update_tree_view_expand_path(
            &folder_tree_store,
            &*pack_file_decoded.borrow(),
            &folder_tree_selection,
            &folder_tree_view,
            true
        );
        Inhibit(false)
    }));


    // When we hit the "Extract file/folder" button.
    tree_view_extract_file.connect_button_release_event(clone!(
        success_dialog,
        error_dialog,
        pack_file_decoded,
        folder_tree_selection,
        context_menu_tree_view => move |_,_|{

        // First, we hide the context menu.
        context_menu_tree_view.popdown();

        let tree_path = ui::get_tree_path_from_selection(&folder_tree_selection, false);

        // Then, we check with the correlation data if the tree_path is a folder or a file.
        let mut is_a_file = false;
        for i in &*pack_file_decoded.borrow().pack_file_data.packed_files {
            if &i.packed_file_path == &tree_path {
                is_a_file = true;
                break;
            }
        }

        // Both (folder and file) are processed in the same way but we need a different
        // FileChooser for files and folders, so we check first what it's.
        if is_a_file {
            file_chooser_extract_file.set_current_name(&tree_path.last().unwrap());
            if file_chooser_extract_file.run() == gtk::ResponseType::Ok.into() {
                match pack_file_manager::extract_from_packfile(
                    &*pack_file_decoded.borrow(),
                    tree_path,
                    file_chooser_extract_file.get_filename().expect("Couldn't open file")) {
                    Ok(result) => {
                        ui::show_dialog(&success_dialog, result);
                    }
                    Err(result) => {
                        ui::show_dialog(&error_dialog, result)
                    }
                }
            }
            file_chooser_extract_file.hide_on_delete();
        }
        else {
            if file_chooser_extract_folder.run() == gtk::ResponseType::Ok.into() {
                match pack_file_manager::extract_from_packfile(
                    &*pack_file_decoded.borrow(),
                    tree_path,
                    file_chooser_extract_folder.get_filename().expect("Couldn't open file")) {
                    Ok(result) => {
                        ui::show_dialog(&success_dialog, result);
                    }
                    Err(result) => {
                        ui::show_dialog(&error_dialog, result)
                    }
                }
            }
            file_chooser_extract_folder.hide_on_delete();
        }

        Inhibit(false)
    }));

    /*
    --------------------------------------------------------
                        Special Events
    --------------------------------------------------------
    */

    // When we double-click something in the TreeView (or click something already selected).
    folder_tree_view.connect_row_activated(clone!(
        error_dialog,
        pack_file_decoded,
        folder_tree_view,
        folder_tree_store,
        folder_tree_selection,
        rename_popover,
        rename_popover_text_entry => move |_,_,_| {

        // First, we get the variable for the new name and spawn the popover.
        let new_name: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

        let rect = ui::get_rect_for_popover(&folder_tree_selection, &folder_tree_view);
        rename_popover.set_pointing_to(&rect);
        rename_popover.popup();

        // Now, in the "New Name" popup, we wait until "Enter" (65293) is hit AND released.
        // In that point, we try to rename the file/folder selected. If we success, the TreeView is
        // updated. If not, we get a Dialog saying why.
        rename_popover.connect_key_release_event(clone!(
            error_dialog,
            pack_file_decoded,
            folder_tree_view,
            folder_tree_store,
            folder_tree_selection,
            rename_popover,
            rename_popover_text_entry,
            new_name => move |_, key| {

            let key_val = key.get_keyval();
            if key_val == 65293 {
                let mut name_changed = false;
                let tree_path = ui::get_tree_path_from_selection(&folder_tree_selection, true);
                *new_name.borrow_mut() = rename_popover_text_entry.get_buffer().get_text();
                match pack_file_manager::rename_packed_file(&mut *pack_file_decoded.borrow_mut(), tree_path.to_vec(), &*new_name.borrow()) {
                    Ok(_) => {
                        rename_popover.popdown();
                        name_changed = true;
                    }
                    Err(result) => {
                        ui::show_dialog(&error_dialog, result);
                    }
                }
                if name_changed {
                    ui::update_tree_view_expand_path(
                        &folder_tree_store,
                        &*pack_file_decoded.borrow(),
                        &folder_tree_selection,
                        &folder_tree_view,
                        true
                    );
                }
                rename_popover_text_entry.get_buffer().set_text("");
            }
            // We need to set this to true to avoid the Enter re-fire this event again and again.
            Inhibit(true)
        }));
        Inhibit(true);
    }));


    // When you select a file in the TreeView, decode it with his codec, if it's implemented.
    folder_tree_view.connect_cursor_changed(clone!(
        pack_file_decoded,
        folder_tree_selection => move |_| {

        // First, we destroy any childrens that the ScrolledWindow we use may have, cleaning it.
        let childrens_to_utterly_destroy =  packed_file_data_display.get_children();
        for i in childrens_to_utterly_destroy.iter() {
            i.destroy();
        }

        // Then, we get the tree_path selected, and check if it's a folder or a file.
        let tree_path = ui::get_tree_path_from_selection(&folder_tree_selection, false);

        let mut is_a_file = false;
        let mut index: i32 = 0;
        for i in &*pack_file_decoded.borrow().pack_file_data.packed_files {
            if i.packed_file_path == tree_path {
                is_a_file = true;
                break;
            }
            index += 1;
        }

        // Only in case it's a file, we do something.
        if is_a_file {

            // First, we get his type to decode it properly
            let mut packed_file_type: &str = "None";
            if tree_path.last().unwrap().ends_with(".loc") {
                packed_file_type = "Loc";
            }

            // Then, depending of his type we decode it properly (if we have it implemented support
            // for his type).
            match packed_file_type {
                "Loc" => {

                    // First, we create the new TreeView and all the needed stuff, and prepare it to
                    // display the data from the Loc file.
                    let packed_file_tree_view_stuff = ui::packed_file_loc::PackedFileLocTreeView::create_tree_view(&packed_file_data_display);
                    let packed_file_tree_view = packed_file_tree_view_stuff.packed_file_tree_view;
                    let packed_file_list_store = packed_file_tree_view_stuff.packed_file_list_store;
                    let packed_file_tree_view_selection = packed_file_tree_view_stuff.packed_file_tree_view_selection;
                    let packed_file_tree_view_cell_key = packed_file_tree_view_stuff.packed_file_tree_view_cell_key;
                    let packed_file_tree_view_cell_text = packed_file_tree_view_stuff.packed_file_tree_view_cell_text;
                    let packed_file_tree_view_cell_tooltip = packed_file_tree_view_stuff.packed_file_tree_view_cell_tooltip;

                    // Then we populate the TreeView with the entries of the Loc PackedFile.
                    let packed_file_data_encoded = &*pack_file_decoded.borrow().pack_file_data.packed_files[index as usize].packed_file_data;
                    let packed_file_data_decoded = Rc::new(RefCell::new(::pack_file_manager::packed_files_manager::loc::Loc::read(packed_file_data_encoded.to_vec())));
                    let mut j = 0;
                    for i in &*packed_file_data_decoded.borrow().packed_file_data.packed_file_data_entries {
                        j += 1;
                        packed_file_list_store.insert_with_values(None, &[0, 1, 2, 3], &[&j.to_string(), &i.key, &i.text, &i.tooltip]);
                    }

                    // Here they come!!! This is what happen when we edit the cells
                    packed_file_tree_view_cell_key.connect_edited(clone!(
                        pack_file_decoded,
                        packed_file_data_decoded,
                        packed_file_tree_view,
                        packed_file_list_store,
                        packed_file_tree_view_selection => move |_,_, new_text|{

                        let edited_cell = packed_file_tree_view_selection.get_selected();
                        let edited_cell_column = packed_file_tree_view.get_cursor();
                        packed_file_list_store.set_value(&edited_cell.unwrap().1, edited_cell_column.1.unwrap().get_sort_column_id() as u32, &new_text.to_value());

                        // Get the data from the table and turn it into a Vec<u8> to write it.
                        packed_file_data_decoded.borrow_mut().packed_file_data = ui::packed_file_loc::PackedFileLocTreeView::return_data_from_tree_view(&packed_file_list_store);
                        ::pack_file_manager::update_packed_file_data(
                            &*packed_file_data_decoded.borrow_mut(),
                            &mut *pack_file_decoded.borrow_mut(),
                            index as usize);
                    }));


                    packed_file_tree_view_cell_text.connect_edited(clone!(
                        pack_file_decoded,
                        packed_file_data_decoded,
                        packed_file_tree_view,
                        packed_file_list_store => move |_,_, new_text|{

                        let edited_cell = packed_file_tree_view_selection.get_selected();
                        let edited_cell_column = packed_file_tree_view.get_cursor();
                        packed_file_list_store.set_value(&edited_cell.unwrap().1, edited_cell_column.1.unwrap().get_sort_column_id() as u32, &new_text.to_value());

                        // Get the data from the table and turn it into a Vec<u8> to write it.
                        packed_file_data_decoded.borrow_mut().packed_file_data = ui::packed_file_loc::PackedFileLocTreeView::return_data_from_tree_view(&packed_file_list_store);
                        ::pack_file_manager::update_packed_file_data(
                            &*packed_file_data_decoded.borrow_mut(),
                            &mut *pack_file_decoded.borrow_mut(),
                            index as usize);
                    }));



                    packed_file_tree_view_cell_tooltip.connect_toggled(clone!(
                        pack_file_decoded,
                        packed_file_data_decoded,
                        packed_file_tree_view,
                        packed_file_list_store => move |cell, tree_path|{

                        let tree_iter = packed_file_list_store.get_iter(&tree_path).unwrap();
                        // Get (Option<TreePath>, Option<TreeViewColumn>)
                        let edited_cell_column: u32 = packed_file_tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;
                        let new_value: bool = packed_file_list_store.get_value(&tree_iter, edited_cell_column as i32).get().unwrap();
                        let new_value_bool = (!new_value).to_value();
                        cell.set_active(!new_value);
                        packed_file_list_store.set_value(&tree_iter, edited_cell_column, &new_value_bool);

                        // Get the data from the table and turn it into a Vec<u8> to write it.
                        packed_file_data_decoded.borrow_mut().packed_file_data = ui::packed_file_loc::PackedFileLocTreeView::return_data_from_tree_view(&packed_file_list_store);
                        ::pack_file_manager::update_packed_file_data(
                            &*packed_file_data_decoded.borrow_mut(),
                            &mut *pack_file_decoded.borrow_mut(),
                            index as usize);
                    }));

                    // When w hit + we add a line. When we hit - we delete the selected line.
                    packed_file_tree_view.connect_key_release_event(clone!(
                        packed_file_tree_view,
                        packed_file_list_store => move |_, key| {

                        let key_val = key.get_keyval();

                        // If we press "+", we add a new line.
                        // NOTE: The new lines have "New" as autogenerated index, to differentiate them.
                        if key_val == 65451 /* + */ {
                            packed_file_list_store.insert_with_values(None, &[0, 1, 2, 3], &[&"New".to_value(), &"New_line".to_value(), &"New_line".to_value(), &false.to_value()]);
                        }

                        // If we press "-", we delete the currently selected line.
                        else if key_val == 65453 /* - */ {

                            // In case we don't have any lines, we do nothing.
                            if packed_file_tree_view.get_cursor().0 != None {
                                let selected_line_tree_path = packed_file_tree_view.get_cursor().0.unwrap();
                                let selected_line_tree_iter = packed_file_list_store.get_iter(&selected_line_tree_path).unwrap();
                                packed_file_list_store.remove(&selected_line_tree_iter);
                            }
                        }
                        Inhibit(true)
                    }));
                }
                // If we reach this point, the coding to implement this type of file is not done yet,
                // so we ignore the file.
                _ => {
                    println!("PackedFile Type not yet implemented.")
                }
            }
        }

        Inhibit(false);
    }));

    // We start GTK. Yay
    gtk::main();
}


