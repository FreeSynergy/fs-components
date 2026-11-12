"""Names the contract may point at. Pattern: none (constants)."""

# WAI-ARIA 1.2, section 5.3: the roles a page can use (abstract roles are left out on purpose).
ARIA_ROLES = frozenset("""
alert alertdialog application article banner blockquote button caption cell checkbox code columnheader combobox
complementary contentinfo definition deletion dialog directory document emphasis feed figure form generic grid gridcell
group heading img insertion link list listbox listitem log main marquee math menu menubar menuitem menuitemcheckbox
menuitemradio meter navigation none note option paragraph presentation progressbar radio radiogroup region row rowgroup
rowheader scrollbar search searchbox separator slider spinbutton status strong subscript superscript switch tab table
tablist tabpanel term textbox time timer toolbar tooltip tree treegrid treeitem
""".split())

# Open UI research list of components (https://open-ui.org/), names lower-cased with dashes.
OPEN_UI_NAMES = frozenset("""
accordion alert avatar badge breadcrumb button card carousel checkbox combobox datepicker dialog disclosure file flex
icon image list menu number-input popup radio-button select skeleton slider switch table tabs tag text text-input toast
tooltip
""".split())

NAMING = ("aria", "open-ui", "own")
SCOPES = ("general", "block", "program")
STATUSES = ("mapped", "provisional", "remove")
NAME_PATTERN = r"[a-z0-9]+(?:-[a-z0-9]+)*"
