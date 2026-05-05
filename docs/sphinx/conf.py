project = "perlchecker"
author = "Daniel Hilst Selli"
version = "0.1.0"
release = "0.1.0"

extensions = [
    "sphinx_copybutton",
]

exclude_patterns = ["_build", ".venv"]

html_theme = "furo"
highlight_language = "perl"
pygments_style = "friendly"
pygments_dark_style = "monokai"

html_title = "perlchecker"
html_static_path = ["_static"]
templates_path = ["_templates"]
