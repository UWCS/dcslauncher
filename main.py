#!/usr/bin/python3
from shutil import which
import json
import subprocess
from typing import List
import requests
from threading import Thread
import gi
gi.require_version('Gtk', '3.0')
from gi.repository import GLib, Gtk, Gdk, GdkPixbuf

from datetime import date

class MyWindow(Gtk.Window):

    def __init__(self):
        Gtk.Window.__init__(self, title="dcspkg GUI")
        self.set_border_width(0)
        self.set_default_size(800, 600)

        window = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        self.add(window)
        if which("dcspkg") is not None:
            hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=5)
            window.pack_end(hbox, False, False, 0)
            self.install_text = Gtk.Label()
            hbox.pack_start(self.install_text, False, False, 10)
            if date.today().weekday() == 0:
                self.install_text.set_label("milk monday moment")
            self.install_button = Gtk.Button()
            self.install_button.connect("clicked", self.on_install)
            hbox.pack_end(self.install_button, False, False, 0)

            main = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=5)
            window.pack_start(main, True, True, 0)

            vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=5)
            main.pack_start(vbox, False, False, 0)

            pixbuf = GdkPixbuf.Pixbuf.new_from_file_at_scale(
                filename="logo_wide.svg",
                width=256,
                height=-1,
                preserve_aspect_ratio=True)
            image = Gtk.Image.new_from_pixbuf(pixbuf)
            image.props.valign = Gtk.Align.START
            vbox.pack_start(image, True, True, 10)

            # Create a scrolled window to hold the list of entries
            scrolled_window = Gtk.ScrolledWindow()
            scrolled_window.set_policy(
                Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
            main.pack_start(scrolled_window, True, True, 0)

            # Create a list box to hold the entries
            self.listbox = Gtk.ListBox()
            self.listbox.set_selection_mode(Gtk.SelectionMode.SINGLE)
            self.listbox.connect('row-activated', self.on_row_activated)
            scrolled_window.add(self.listbox)

            self.job = None
            self.pkgs = dcspkg_json("list")
            self.pkgs_installed = dcspkg_json("installed")

            for package in self.pkgs:
                hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=0)
                self.listbox.add(hbox)

                image = Gtk.Image.new_from_icon_name("image-x-generic", 6)
                image.set_size_request(128, 128)
                hbox.pack_start(image, False, False, 0)
                load_image_threaded(image, package['image_url'])

                vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
                hbox.pack_start(vbox, False, False, 0)

                label = Gtk.Label()
                label.set_markup(
                    f"<span size=\"x-large\" weight=\"bold\">{package['fullname']}</span>")
                label.props.halign = Gtk.Align.START
                vbox.pack_start(label, False, False, 0)

                label = Gtk.Label(label=package['description'])
                label.set_justify(Gtk.Justification.LEFT)
                label.set_line_wrap(True)
                # Deprecated but halign dont do anything lol
                label.set_alignment(0, 0)
                vbox.pack_start(label, False, False, 0)
        else:
            self.job = None
            label = Gtk.Label(label="dcspkg is not installed, run cargo install dcspkg after installing rust")
            window.pack_start(label, True, True, 0)

    def on_row_activated(self, listbox, row):
        pkg_name = self.pkgs[row.get_index()]['pkgname']
        if any(pkg['pkgname'] == pkg_name for pkg in self.pkgs_installed):
            self.install_button.set_label("Play Game")
        else:
            self.install_button.set_label("Install")

    def on_install(self, button):
        def threaded(pkgname, name):
            message = run_command(['dcspkg', 'install', pkgname])
            print(message)
            if not message.startswith("Error"):
                GLib.idle_add(self.install_text.set_label,
                              f"Installed {name}!")
                self.pkgs_installed = dcspkg_json("installed")
                GLib.idle_add(self.on_row_activated, self.listbox,
                              self.listbox.get_selected_row())
            else:
                GLib.idle_add(self.install_text.set_label,
                              f"Encountered error Installing {name}...")
            self.job = None
            GLib.idle_add(self.install_button.set_sensitive, True)
        index = self.listbox.get_selected_row().get_index()
        name = self.pkgs[index]['fullname']
        pkgname = self.pkgs[index]['pkgname']
        if any(pkg['pkgname'] == pkgname for pkg in self.pkgs_installed):
            subprocess.Popen(['dcspkg', 'run', pkgname])
        else:
            button.set_sensitive(False)
            self.job = f"Installing {name}..."
            thread = Thread(target=threaded, args=[pkgname, name])
            self.install_text.set_label(self.job)
            thread.daemon = True
            thread.start()

    def quit(self, _1, _2):
        if self.job == None:
            Gtk.main_quit()
            return False
        return True


def load_image_threaded(imageObj: Gtk.Image, url: str):
    def url2pixbuf(url: str) -> GdkPixbuf:
        response = requests.get(url)
        content = response.content
        loader = GdkPixbuf.PixbufLoader()
        loader.write_bytes(GLib.Bytes.new(content))
        loader.close()
        return loader.get_pixbuf()
    if url != "":
        imageObj.set_from_icon_name("image-loading", 6)

        def threaded(imageObj: Gtk.Image, url: str):
            try:
                pixbuf = url2pixbuf(url)
                pixbuf = pixbuf.scale_simple(
                    128, 128, GdkPixbuf.InterpType.BILINEAR)
                GLib.idle_add(imageObj.set_from_pixbuf, pixbuf)
            except:
                GLib.idle_add(imageObj.set_from_icon_name, "image-missing", 6)
                print(f"Error Loading Image at: {url}")
        thread = Thread(target=threaded, args=(imageObj, url))
        thread.daemon = True
        thread.start()


def run_command(command: List[str]) -> str:
    return subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT).stdout.decode('utf-8')


def dcspkg_json(command: str) -> List[dict]:
    packages = run_command(['dcspkg', command, '-j'])
    packages = json.loads(packages)
    return packages


if __name__ == "__main__":
    win = MyWindow()
    win.connect("delete-event", win.quit)
    win.show_all()
    try:
        win.listbox.unselect_all()
    except:
        pass
    Gtk.main()
