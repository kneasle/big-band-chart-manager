import os
import argparse
from pathlib import Path

import tkinter as tk
import pypdf
import pdf2image
from PIL import ImageTk


def main():
    # Parse command line arguments
    parser = argparse.ArgumentParser(description="Split a PDF into separate charts")
    parser.add_argument("file", help="Path to the PDF file to split")
    parser.add_argument(
        "--output-folder",
        "-o",
        help="Output folder for separated charts (default: Separated Charts/<filename>)",
    )
    args = parser.parse_args()

    # Read args
    file = Path(args.file)
    if args.output_folder is not None:
        output_folder = Path(args.output_folder)
    else:
        # Default output folder based on the input filename
        filename_without_ext = file.stem
        output_folder = Path("Separated Charts") / filename_without_ext

    # Convert the required PDF(s)
    if not output_folder.exists():
        output_folder.mkdir(parents=True, exist_ok=True)

    splitter = SplitterWindow(file, output_folder)
    splitter.run()


# fmt: off
CODE_TO_PART = {
    # Conductor
    "c" : "Conductor",
    # Vocal
    "v" : "Vocal",
    # Saxes
    "s1": "Alto Sax 1",
    "s2": "Alto Sax 2",
    "s3": "Tenor Sax 1",
    "s4": "Tenor Sax 2",
    "s5": "Baritone Sax",
    # Trumpets
    "t1": "Trumpet 1",
    "t2": "Trumpet 2",
    "t3": "Trumpet 3",
    "t4": "Trumpet 4",
    "t5": "Trumpet 5",
    # Trombones
    "b1": "Trombone 1",
    "b2": "Trombone 2",
    "b3": "Trombone 3",
    "b4": "Trombone 4",
    "b5": "Trombone 5",
    # Rhythm Section
    "g" : "Guitar",
    "p" : "Piano",
    "b" : "Bass",
    "d" : "Drums",
}
# fmt: on


class SplitterWindow:
    def __init__(self, input_file: Path, output_folder: Path):
        self.input_file: Path = input_file
        self.output_folder: Path = output_folder

        # Create TK stuff
        self.root: tk.Tk = tk.Tk()
        self.root.title("PDF splitter")
        self.status_label: tk.Label = tk.Label(self.root, text="Status...", font=("Ariel", 20))
        self.status_label.pack(padx=10, pady=10)
        self.entry: tk.Entry = tk.Entry(self.root, width=50)
        self.entry.pack(padx=10, pady=(0, 10))
        self.entry.focus_set()
        self.next_part_label: tk.Label = tk.Label(
            self.root, text="Next part line...", font=("Ariel", 20)
        )
        self.next_part_label.pack(padx=10, pady=10)
        self.image_label: tk.Label = tk.Label(self.root)
        self.image_label.pack(padx=10, pady=10)

        # Bind Enter key
        self.entry.bind("<Return>", self.on_enter)

        # Read input PDF to images (after the TK window is created)
        raw_page_images = pdf2image.convert_from_path(input_file, dpi=80)
        self.page_images: list[ImageTk.PhotoImage] = [
            ImageTk.PhotoImage(image) for image in raw_page_images
        ]

        # Read pages
        self.pdf_reader = pypdf.PdfReader(self.input_file)
        assert len(self.pdf_reader.pages) == len(self.page_images)

        # Initialise GUI
        self.page_idx: int = 0
        self.current_part: str | None = None
        self.pages_in_current_part: list[pypdf.PageObject] = []
        self.update_gui()

    def run(self):
        # Run the loop, which will close once all pages are sorted
        self.root.mainloop()

    def on_enter(self, event=None):
        # Handle input
        user_text = self.entry.get()
        if user_text == "":
            self.move_to_next_image()
        elif next_part := self.read_next_part(user_text):
            self.start_new_part(next_part)
            self.move_to_next_image()
        else:
            print(f"Unknown input: {repr(user_text)}")

        if self.page_idx >= len(self.page_images):
            # If we've run out of pages, save the last part and exit
            self.save_current_part()
            self.root.destroy()
        else:
            # Update the GUI for the next page
            self.entry.delete(0, tk.END)
            self.update_gui()

    def read_next_part(self, user_text: str) -> str | None:
        # Custom part name
        if user_text.startswith("o: "):
            return user_text.removeprefix("o: ")

        # Void (destroy PDF pages)
        if user_text == "void":
            return "void"

        # 'n' for next
        if user_text == "n":
            next_part = self.infer_next_part()
            assert next_part is not None
            return next_part

        # Part code
        if user_text in CODE_TO_PART:
            return CODE_TO_PART[user_text]

    def move_to_next_image(self):
        # TODO: Save current page
        self.pages_in_current_part.append(self.pdf_reader.pages[self.page_idx])

        # Move to next page
        self.page_idx += 1

    def start_new_part(self, new_part_name: str):
        self.save_current_part()

        # Start a fresh new part
        self.current_part = new_part_name
        self.pages_in_current_part = []

    def save_current_part(self):
        if self.current_part is None or self.current_part == "void":
            return  # No part to save

        # Get file location
        part_filename = f"{self.current_part} - {self.output_folder.name}.pdf"
        part_path = self.output_folder / part_filename
        print(f"Saving part to {part_path}")

        # Save the part
        pdf_writer = pypdf.PdfWriter()
        for page in self.pages_in_current_part:
            pdf_writer.add_page(page)
        pdf_writer.write(part_path)

    def update_gui(self):
        self.image_label.config(image=self.page_images[self.page_idx])
        self.status_label.config(text=self.get_statusline())
        self.next_part_label.config(text=self.get_next_part_line())

    def get_statusline(self):
        statusline = (
            f"Splitting '{self.input_file}', page {self.page_idx + 1}/{len(self.page_images)}:\n"
        )

        if self.current_part is None:
            statusline += "No part set yet..."
        elif self.current_part == "void":
            statusline += f"Voiding {len(self.pages_in_current_part)} pages"
        else:
            statusline += (
                f"Current part: '{self.current_part}' ({len(self.pages_in_current_part)} pages)"
            )

        return statusline

    def get_next_part_line(self):
        next_part = self.infer_next_part()
        if next_part is None:
            return "No next part inferred."
        else:
            return f"Type 'n' for '{next_part}'."

    def infer_next_part(self) -> str | None:
        ORDERING = [
            # Conductor
            "Conductor",
            # Saxes
            "Alto Sax 1",
            "Alto Sax 2",
            "Tenor Sax 1",
            "Tenor Sax 2",
            "Baritone Sax",
            # Trumpets
            "Trumpet 1",
            "Trumpet 2",
            "Trumpet 3",
            "Trumpet 4",
            # Trombones
            "Trombone 1",
            "Trombone 2",
            "Trombone 3",
            "Trombone 4",
            # Rhythm Section
            "Piano",
            "Guitar",
            "Bass",
            "Drums",
        ]

        SPECIAL_ORDERING = {
            "Trumpet 5": "Trombone 1",
            "Trombone 5": "Piano",
        }

        if self.current_part is None:
            return ORDERING[0]

        if self.current_part in ORDERING:
            index = ORDERING.index(self.current_part) + 1
            return None if index == len(ORDERING) else ORDERING[index]

        if self.current_part in SPECIAL_ORDERING:
            return SPECIAL_ORDERING[self.current_part]

        return None


if __name__ == "__main__":
    main()
