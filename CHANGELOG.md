# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-09-20

### Added
- **Core MLA 9 Architecture**:
  - Locked 1.0-inch margins, 2.0 double-spacing, 12 pt academic serif typefaces.
  - Automatic 0.5-inch first-line paragraph indentation.
  - First-page heading block (Student Name, Instructor, Course, Date).
  - Running header `[LastName] [PageNumber]` positioned 0.5 in from top edge.
  - Centered document title with intelligent MLA Title Case converter.
  - Dedicated Block Quote blocks (>4 lines) with 0.5 in left indentation.
  - MLA Section Headings (Level 1 Bold, Level 2 Italics, Level 3 Centered Bold).
- **Works Cited Manager**:
  - Full support for the MLA 9th Edition 9 Core Elements container model.
  - Automatic alphabetical sorting by author or primary title.
  - True 0.5-inch hanging indent formatting.
  - Real-time formatted preview with italicized containers and quotation-wrapped short works.
- **In-Text Citation Assistant**:
  - Quick insertion dialog linked directly to the Works Cited entries.
  - Generates parenthetical citations (e.g. `(Morrison 42)`).
- **MLA 9 Compliance Inspector**:
  - Real-time document linter checking structure, formatting, punctuation, and citations.
  - Live compliance score badge (0-100%).
  - 1-click Auto-Fix capabilities.
- **Frosted Glass Transparency & Customization**:
  - Native OS blur/vibrancy: Windows 11 Mica, Windows 10/11 Acrylic, macOS Vibrancy.
  - Transparent writing paper sheet with independent opacity slider.
  - Presets: Frosted Obsidian, Frosted Parchment, Nordic Frost, Amber Glass, and Custom RGB palettes.
- **Custom Keybindings & Live Shortcut Preview**:
  - Remappable shortcuts for all 15 editor actions with full GUI customizer.
  - Live keybind preview sidebar on the left displaying custom shortcuts with styled keycap badges.
  - Responsive layout: automatically collapses and hides the preview panel when the window is too narrow to fit both the sidebar and the manuscript page.
- **Distraction-Free Zen Mode**:
  - Fullscreen writing mode with multiple instant exit safeguards: `Escape` key, `F11`, and a floating frosted exit button.
- **High-Fidelity Document Exporters**:
  - Microsoft Word `.docx` exporter with exact MLA margin and spacing twips.
  - Print-ready HTML exporter with embedded `@page` CSS for 1-click PDF printing.
  - Plain text / Markdown exporter with standardized MLA indentations.
- **Packaging & Distribution**:
  - Standalone testing/portable build mode (`build_portable.ps1`, `build_portable.sh`).
  - Windows Inno Setup installer script with desktop/start-menu shortcuts and `.mladoc` file associations.
  - macOS DMG packager script and `.app` bundle definition (`Info.plist`).
  - Linux Debian `.deb` package script and `.desktop` entry.
