Lists

Introduction

	Lists organize related items in sequence. They are collections of at least two list items, distinguished from single-item paragraphs.

Syntax

	Pattern:
		<blank-line>
		- First item
		- Second item
		- Third item

	Key rule: Lists REQUIRE a preceding blank line
	(for disambiguation from paragraphs containing dash-prefixed text)

	Minimum items: 2
	(single dash-prefixed lines are paragraphs, not lists)

List Item Markers

	Plain (unordered):
		- Item text here

	Numbered:
		1. First item
		2. Second item

	Alphabetical:
		a. First item
		b. Second item

	Parenthetical:
		1) First item
		2) Second item
		a) Alphabetical with parens

	Roman numerals:
		I. First item
		II. Second item

Mixing Markers

	List items can mix different marker styles within the same list.
	The first item's style sets the semantic type, but rendering is flexible.

	Example (all treated as single list):
		1. First item
		2. Second item
		a. Third item
		- Fourth item

Blank Line Rule

	Lists require a preceding blank line for disambiguation:

	Paragraph (no list):
		Some text
		- This dash is just text, not a list item

	List (has blank line):
		Some text

		- This is a list item
		- Second item

	No blank lines BETWEEN list items:
		- Item one
		- Item two
		
		- This starts a NEW list (blank line terminates previous)

Content

	List items contain text on the same line as the marker.
	Indented content can contain:
		- Paragraphs (multiple paragraphs allowed)
		- Nested lists (list-in-list nesting)
		- Mix of paragraphs and nested lists
	List items CANNOT contain:
		- Sessions (use definitions instead for titled containers)
		- Annotations (inline or block)

Block Termination

	Lists end on:
		- Blank line (creates gap to next element)
		- Dedent (back to parent level)
		- End of document
		- Start of new element at same/lower indent level

Examples

	Simple unordered list:
		- Apples
		- Bananas
		- Oranges

	Numbered list:
		1. First step
		2. Second step
		3. Third step

	Mixed markers:
		1. Introduction
		2. Main content
		a. Subsection A
		b. Subsection B
		3. Conclusion

	Lists in definitions:
		HTTP Methods:
		    - GET: Retrieve resources
		    - POST: Create resources
		    - PUT: Update resources

	Multiple lists in sequence:
		List one:

		- Item A
		- Item B

		List two:

		- Item X
		- Item Y

	List items with nested paragraphs:
		1. Introduction
		    This is a paragraph nested inside the first list item.

		- Key point
		    Supporting details for this key point.

		    Additional context paragraph.

	List items with mixed content:
		- First item
		    Opening paragraph.

		    - Nested list item one
		    - Nested list item two

		    Closing paragraph.

Use Cases

	- Task lists and checklists
	- Enumerated steps or instructions
	- Feature lists
	- Options or choices
	- Bulleted information
	- Ordered sequences
