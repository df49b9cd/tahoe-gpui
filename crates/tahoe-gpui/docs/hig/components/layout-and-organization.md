## Components › Layout and organization

The Layout and organization subcategory covers components that structure and present content: Boxes for visually grouping related content, Collections for grid and list layouts, Column views for hierarchical navigation, Disclosure controls for showing and hiding content, Labels for static text, Lists and tables for row-based content, Lockups for tvOS media presentation, Outline views for hierarchical data, Split views for multi-pane layouts, and Tab views for switching between content panes.

### Section map

| Page | Canonical URL |
|---|---|
| Boxes | https://developer.apple.com/design/human-interface-guidelines/boxes |
| Collections | https://developer.apple.com/design/human-interface-guidelines/collections |
| Column views | https://developer.apple.com/design/human-interface-guidelines/column-views |
| Disclosure controls | https://developer.apple.com/design/human-interface-guidelines/disclosure-controls |
| Labels | https://developer.apple.com/design/human-interface-guidelines/labels |
| Lists and tables | https://developer.apple.com/design/human-interface-guidelines/lists-and-tables |
| Lockups | https://developer.apple.com/design/human-interface-guidelines/lockups |
| Outline views | https://developer.apple.com/design/human-interface-guidelines/outline-views |
| Split views | https://developer.apple.com/design/human-interface-guidelines/split-views |
| Tab views | https://developer.apple.com/design/human-interface-guidelines/tab-views |

### Detailed pages

---

### Boxes
**Path:** Components › Layout and organization
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/boxes

#### Hero image
![Boxes](../images/components-box-intro@2x.png)
*A stylized representation of a group of interface elements within a rounded rectangle. The image is tinted red to subtly reflect the red in the original six-color Apple logo.*

#### Summary
A box creates a visually distinct group of logically related information and components.

By default, a box uses a visible border or background color to separate its contents from the rest of the interface. A box can also include a title.

#### Best practices

Prefer keeping a box relatively small in comparison with its containing view. As a box's size gets close to the size of the containing window or screen, it becomes less effective at communicating the separation of grouped content, and it can crowd other content.

Consider using padding and alignment to communicate additional grouping within a box. A box's border is a distinct visual element — adding nested boxes to define subgroups can make your interface feel busy and constrained.

#### Content

Provide a succinct introductory title if it helps clarify the box's contents. The appearance of a box helps people understand that its contents are related, but it might make sense to provide more detail about the relationship. Also, a title can help VoiceOver users predict the content they encounter within the box.

If you need a title, write a brief phrase that describes the contents. Use sentence-style capitalization. Avoid ending punctuation unless you use a box in a settings pane, where you append a colon to the title.

#### Platform considerations

No additional considerations for visionOS. Not supported in tvOS or watchOS.

**iOS, iPadOS**
By default, iOS and iPadOS use the secondary and tertiary background colors in boxes.

**macOS**
By default, macOS displays a box's title above it.

#### Resources

**Related**
- Layout

**Developer documentation**
- GroupBox — SwiftUI
- NSBox — AppKit

---

### Collections
**Path:** Components › Layout and organization
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/collections

#### Hero image
![Collections](../images/components-collection-view-intro@2x.png)
*A stylized representation of eight image icons, separated into two rows of four. The image is tinted red to subtly reflect the red in the original six-color Apple logo.*

#### Summary
A collection manages an ordered set of content and presents it in a customizable and highly visual layout.

Generally speaking, collections are ideal for showing image-based content.

#### Best practices

Use the standard row or grid layout whenever possible. Collections display content by default in a horizontal row or a grid, which are simple, effective appearances that people expect. Avoid creating a custom layout that might confuse people or draw undue attention to itself.

Consider using a table instead of a collection for text. It's generally simpler and more efficient to view and digest textual information when it's displayed in a scrollable list.

Make it easy to choose an item. If it's too difficult to get to an item in your collection, people will get frustrated and lose interest before reaching the content they want. Use adequate padding around images to keep focus or hover effects easy to see and prevent content from overlapping.

Add custom interactions when necessary. By default, people can tap to select, touch and hold to edit, and swipe to scroll. If your app requires it, you can add more gestures for performing custom actions.

Consider using animations to provide feedback when people insert, delete, or reorder items. Collections support standard animations for these actions, and you can also use custom animations.

#### Platform considerations

No additional considerations for macOS, tvOS, or visionOS. Not supported in watchOS.

**iOS, iPadOS**
Use caution when making dynamic layout changes. The layout of a collection can change dynamically. Be sure any changes make sense and are easy to track. If possible, try to avoid changing the layout while people are viewing and interacting with it, unless it's in response to an explicit action.

#### Resources

**Related**
- Lists and tables
- Image views
- Layout

**Developer documentation**
- UICollectionView — UIKit
- NSCollectionView — AppKit

---

### Column views
**Path:** Components › Layout and organization
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/column-views

#### Hero image
![Column views](../images/components-column-view-intro@2x.png)
*A stylized representation of three columns containing a list of folders, images, and file information. The image is tinted red to subtly reflect the red in the original six-color Apple logo.*

#### Summary
A column view — also called a browser — lets people view and navigate a data hierarchy using a series of vertical columns.

Each column represents one level of the hierarchy and contains horizontal rows of data items. Within a column, any parent item that contains nested child items is marked with a triangle icon. When people select a parent, the next column displays its children. People can continue navigating in this way until they reach an item with no children, and can also navigate back up the hierarchy to explore other branches of data.

> Note If you need to manage the presentation of hierarchical content in your iPadOS or visionOS app, consider using a split view.

#### Best practices

Consider using a column view when you have a deep data hierarchy in which people tend to navigate back and forth frequently between levels, and you don't need the sorting capabilities that a list or table provides. For example, Finder offers a column view (in addition to icon, list, and gallery views) for navigating directory structures.

Show the root level of your data hierarchy in the first column. People know they can quickly scroll back to the first column to begin navigating the hierarchy from the top again.

Consider showing information about the selected item when there are no nested items to display. The Finder, for example, shows a preview of the selected item and information like the creation date, modification date, file type, and size.

Let people resize columns. This is especially important if the names of some data items are too long to fit within the default column width.

#### Platform considerations

Not supported in iOS, iPadOS, tvOS, visionOS, or watchOS.

#### Resources

**Related**
- Lists and tables
- Outline views
- Split views

**Developer documentation**
- NSBrowser — AppKit

---

### Disclosure controls
**Path:** Components › Layout and organization
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/disclosure-controls

#### Hero image
![Disclosure controls](../images/components-disclosure-control-intro@2x.png)
*A stylized representation of collapsed and expanded disclosure buttons. The image is tinted red to subtly reflect the red in the original six-color Apple logo.*

#### Summary
Disclosure controls reveal and hide information and functionality related to specific controls or views.

#### Best practices

Use a disclosure control to hide details until they're relevant. Place controls that people are most likely to use at the top of the disclosure hierarchy so they're always visible, with more advanced functionality hidden by default. This organization helps people quickly find the most essential information without overwhelming them with too many detailed options.

#### Disclosure triangles

A disclosure triangle shows and hides information and functionality associated with a view or a list of items. For example, Keynote uses a disclosure triangle to show advanced options when exporting a presentation, and the Finder uses disclosure triangles to progressively reveal hierarchy when navigating a folder structure in list view.

A disclosure triangle points inward from the leading edge when its content is hidden and down when its content is visible. Clicking or tapping the disclosure triangle switches between these two states, and the view expands or collapses accordingly to accommodate the content.

Provide a descriptive label when using a disclosure triangle. Make sure your labels indicate what is disclosed or hidden, like "Advanced Options."

For developer guidance, see NSButton.BezelStyle.disclosure.

#### Disclosure buttons

A disclosure button shows and hides functionality associated with a specific control. For example, the macOS Save sheet shows a disclosure button next to the Save As text field. When people click or tap this button, the Save dialog expands to give advanced navigation options for selecting an output location for their document.

A disclosure button points down when its content is hidden and up when its content is visible. Clicking or tapping the disclosure button switches between these two states, and the view expands or collapses accordingly to accommodate the content.

Place a disclosure button near the content that it shows and hides. Establish a clear relationship between the control and the expanded choices that appear when a person clicks or taps a button.

Use no more than one disclosure button in a single view. Multiple disclosure buttons add complexity and can be confusing.

For developer guidance, see NSButton.BezelStyle.pushDisclosure.

#### Platform considerations

No additional considerations for macOS. Not supported in tvOS or watchOS.

**iOS, iPadOS, visionOS**
Disclosure controls are available in iOS, iPadOS, and visionOS with the SwiftUI DisclosureGroup view.

#### Resources

**Related**
- Outline views
- Lists and tables
- Buttons

**Developer documentation**
- DisclosureGroup — SwiftUI
- NSButton.BezelStyle.disclosure — AppKit
- NSButton.BezelStyle.pushDisclosure — AppKit

**Videos**
- Stacks, Grids, and Outlines in SwiftUI

---

### Labels
**Path:** Components › Layout and organization
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/labels

#### Hero image
![Labels](../images/components-label-intro@2x.png)
*A stylized representation of a text label. The image is tinted red to subtly reflect the red in the original six-color Apple logo.*

#### Summary
A label is a static piece of text that people can read and often copy, but not edit.

Labels display text throughout the interface, in buttons, menu items, and views, helping people understand the current context and what they can do next.

The term label refers to uneditable text that can appear in various places. For example:
- Within a button, a label generally conveys what the button does, such as Edit, Cancel, or Send.
- Within many lists, a label can describe each item, often accompanied by a symbol or an image.
- Within a view, a label might provide additional context by introducing a control or describing a common action or task that people can perform in the view.

> Developer note To display uneditable text, SwiftUI defines two components: Label and Text.

The guidance below can help you use a label to display text. In some cases, guidance for specific components — such as action buttons, menus, and lists and tables — includes additional recommendations for using text.

#### Best practices

Use a label to display a small amount of text that people don't need to edit. If you need to let people edit a small amount of text, use a text field. If you need to display a large amount of text, and optionally let people edit it, use a text view.

Prefer system fonts. A label can display plain or styled text, and it supports Dynamic Type (where available) by default. If you adjust the style of a label or use custom fonts, make sure the text remains legible.

Use system-provided label colors to communicate relative importance. The system defines four label colors that vary in appearance to help you give text different levels of visual importance. For additional guidance, see Color.

| System color | Example usage | iOS, iPadOS, tvOS, visionOS | macOS |
|---|---|---|---|
| Label | Primary information | label | labelColor |
| Secondary label | A subheading or supplemental text | secondaryLabel | secondaryLabelColor |
| Tertiary label | Text that describes an unavailable item or behavior | tertiaryLabel | tertiaryLabelColor |
| Quaternary label | Watermark text | quaternaryLabel | quaternaryLabelColor |

Make useful label text selectable. If a label contains useful information — like an error message, a location, or an IP address — consider letting people select and copy it for pasting elsewhere.

#### Platform considerations

No additional considerations for iOS, iPadOS, tvOS, or visionOS.

**macOS**
> Developer note To display uneditable text in a label, use the isEditable property of NSTextField.

**watchOS**
Date and time text components display the current date, the current time, or a combination of both. You can configure a date text component to use a variety of formats, calendars, and time zones. A countdown timer text component displays a precise countdown or count-up timer. You can configure a timer text component to display its count value in a variety of formats.

When you use the system-provided date and timer text components, watchOS automatically adjusts the label's presentation to fit the available space. The system also updates the content without further input from your app.

Consider using date and timer components in complications. For design guidance, see Complications; for developer guidance, see Text.

#### Resources

**Related**
- Text fields
- Text views

**Developer documentation**
- Label — SwiftUI
- Text — SwiftUI
- UILabel — UIKit
- NSTextField — AppKit

#### Change log

| Date | Changes |
|---|---|
| June 5, 2023 | Updated guidance to reflect changes in watchOS 10. |

---

### Lists and tables
**Path:** Components › Layout and organization
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/lists-and-tables

#### Hero image
![Lists and tables](../images/components-lists-and-tables-intro@2x.png)
*A stylized representation of a list view with multiple rows. The image is tinted red to subtly reflect the red in the original six-color Apple logo.*

#### Summary
A list or table presents data as a scrollable sequence of rows, each of which can display content, controls, or both.

Lists are ideal for displaying a large amount of text-based content. Tables support multiple columns and can present related data in a way that's easy to compare.

#### Best practices

Use a list or table when you need to display a sequential series of items. Lists and tables efficiently display large amounts of data in a way that's easy for people to understand and navigate.

Consider using a collection instead when items are primarily images. Collections display image-based content more effectively than lists.

Prefer a table over a list when your data has structure that benefits from multiple columns. Use a list when items are simple and a single column of content is sufficient.

#### Content

Make sure each row has a clear purpose. If a row has multiple pieces of information, establish a clear visual hierarchy by using typographic contrast, images, and spacing.

Use concise row content. Keep row content short enough to fit comfortably within the row height. If content is longer than the available space, either truncate it or display it in multiple lines.

Consider offering an index when displaying a long list. An index lets people quickly jump to a specific section.

#### Style

Use a consistent style for all rows in a list. Mixing different row styles within a list creates a disjointed appearance. If you need to differentiate certain rows, use other visual cues like color or icons.

#### Platform considerations

**iOS, iPadOS, visionOS**
In iOS and iPadOS, lists present rows in a scrollable, single-column format. Rows can include accessory views, swipe actions, and reorder controls.

**macOS**
In macOS, tables present data in a multi-column grid. Tables support sorting, selection, and inline editing.

**tvOS**
Lists present rows in a scrollable format optimized for remote control navigation.

**watchOS**
In watchOS, use lists to display scrollable content. Keep list content concise and focused.

#### Resources

**Related**
- Collections
- Outline views
- Column views

**Developer documentation**
- List — SwiftUI
- UITableView — UIKit
- NSTableView — AppKit

#### Change log

| Date | Changes |
|---|---|
| June 21, 2023 | Updated content guidance. |
| June 5, 2023 | Added guidance for lists and tables in watchOS 10. |

---

### Lockups
**Path:** Components › Layout and organization
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/lockups

#### Hero image
![Lockups](../images/components-lockups-intro@2x.png)
*A stylized representation of a lockup component showing a poster with a title. The image is tinted red to subtly reflect the red in the original six-color Apple logo.*

#### Summary
A lockup combines an image with associated text to create a visually cohesive content item.

Lockups are primarily used in tvOS to present media content — such as movies, TV shows, and music — in a consistent, visually appealing way. Each lockup type is optimized for a specific kind of content.

#### Best practices

Use the lockup type that best suits your content. Choose cards for editorial content, caption buttons for actionable items with labels below, monograms for person-based content, and posters for movie and TV content.

Keep lockup content consistent within a row. When you present multiple lockups together, use the same lockup type and size for all of them. Consistent sizing makes it easier for people to scan the content.

#### Cards

Cards present editorial content with a title, subtitle, and an image or video.

#### Caption buttons

Caption buttons present an image or video with a title and optional subtitle below it. They're well-suited for actionable items in horizontally scrolling rows.

Caption buttons have a label below the image, which makes them easy to identify when navigating with the remote.

#### Monograms

Monograms present a person or character with their name displayed in or below a circular image. Use monograms when showing cast members, creators, or other people.

Use consistent monogram sizes within a row. Mixing sizes creates a disjointed appearance and makes it harder for people to navigate.

#### Posters

Posters present content in a vertical format, like a movie poster. They typically show an image with a title below it.

Use poster lockups for movie and TV show content. Posters follow the standard aspect ratio used for movie posters, making them instantly recognizable.

#### Platform considerations

Not supported in iOS, iPadOS, macOS, visionOS, or watchOS.

#### Resources

**Related**
- Designing for tvOS
- Layout

**Developer documentation**
- TVLockupView — TVUIKit
- TVLockupHeaderFooterView — TVUIKit

**Videos**
- Design for spatial user interfaces

---

### Outline views
**Path:** Components › Layout and organization
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/outline-views

#### Hero image
![Outline views](../images/components-outline-view-intro@2x.png)
*A stylized representation of a list of folders and images, displayed in an outline view containing four columns: [Name], [Date Modified], [Size], and [Kind]. The image is tinted red to subtly reflect the red in the original six-color Apple logo.*

#### Summary
An outline view presents hierarchical data in a scrolling list of cells that are organized into columns and rows.

An outline view includes at least one column that contains primary hierarchical data, such as a set of parent containers and their children. You can add columns, as needed, to display attributes that supplement the primary data; for example, sizes and modification dates. Parent containers have disclosure triangles that expand to reveal their children.

Finder windows offer an outline view for navigating the file system.

#### Best practices

Outline views work well to display text-based content and often appear in the leading side of a split view, with related content on the opposite side.

Use a table instead of an outline view to present data that's not hierarchical. For guidance, see Lists and tables.

Expose data hierarchy in the first column only. Other columns can display attributes that apply to the hierarchical data in the primary column.

Use descriptive column headings to provide context. Use nouns or short noun phrases with title-style capitalization and no punctuation; in particular, avoid adding a trailing colon. Always provide column headings in a multi-column outline view. If you don't include a column heading in a single-column outline view, use a label or other means to make sure there's enough context.

Consider letting people click column headings to sort an outline view. In a sortable outline view, people can click a column heading to perform an ascending or descending sort based on that column. You can implement additional sorting based on secondary columns behind the scenes, if necessary. If people click the primary column heading, sorting occurs at each hierarchy level. For example, in the Finder, all top-level folders are sorted, then the items within each folder are sorted. If people click the heading of a column that's already sorted, the folders and their contents are sorted again in the opposite direction.

Let people resize columns. Data displayed in an outline view often varies in width. It's important to let people adjust column width as needed to reveal data that's wider than the column.

Make it easy for people to expand or collapse nested containers. For example, clicking a disclosure triangle for a folder in a Finder window expands only that folder. However, Option-clicking the disclosure triangle expands all of its subfolders.

Retain people's expansion choices. If people expand various levels of an outline view to reach a specific item, store the state so you can display it again the next time. This way, people won't need to navigate back to the same place again.

Consider using alternating row colors in multi-column outline views. Alternating colors can make it easier for people to track row values across columns, especially in wide outline views.

Let people edit data if it makes sense in your app. In an editable outline view cell, people expect to be able to single-click a cell to edit its contents. Note that a cell can respond differently to a double click. For example, an outline view listing files might let people single-click a file's name to edit it, but double-click a file's name to open the file. You can also let people reorder, add, and remove rows if it would be useful.

Consider using a centered ellipsis to truncate cell text instead of clipping it. An ellipsis in the middle preserves the beginning and end of the cell text, which can make the content more distinct and recognizable than clipped text.

Consider offering a search field to help people find values quickly in a lengthy outline view. Windows with an outline view as the primary feature often include a search field in the toolbar. For guidance, see Search fields.

#### Platform considerations

Not supported in iOS, iPadOS, tvOS, visionOS, or watchOS.

#### Resources

**Related**
- Column views
- Lists and tables
- Split views

**Developer documentation**
- OutlineGroup — SwiftUI
- NSOutlineView — AppKit

**Videos**
- Stacks, Grids, and Outlines in SwiftUI

---

### Split views
**Path:** Components › Layout and organization
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/split-views

#### Hero image
![Split views](../images/components-split-view-intro@2x.png)
*A stylized representation of a window consisting of three areas: a sidebar, a canvas, and an inspector. The image is tinted red to subtly reflect the red in the original six-color Apple logo.*

#### Summary
A split view manages the presentation of multiple adjacent panes of content, each of which can contain a variety of components, including tables, collections, images, and custom views.

Typically, you use a split view to show multiple levels of your app's hierarchy at once and support navigation between them. In this scenario, selecting an item in the view's primary pane displays the item's contents in the secondary pane. Similarly, a split view can display a tertiary pane if items in the secondary pane contain additional content.

It's common to use a split view to display a sidebar for navigation, where the leading pane lists the top-level items or collections in an app, and the secondary and optional tertiary panes can present child collections and item details. Rarely, you might also use a split view to provide groups of functionality that supplement the primary view — for example, Keynote in macOS uses split view panes to present the slide navigator, the presenter notes, and the inspector pane in areas that surround the main slide canvas.

#### Best practices

To support navigation, persistently highlight the current selection in each pane that leads to the detail view. The selected appearance clarifies the relationship between the content in various panes and helps people stay oriented.

Consider letting people drag and drop content between panes. Because a split view provides access to multiple levels of hierarchy, people can conveniently move content from one part of your app to another by dragging items to different panes. For guidance, see Drag and drop.

#### Platform considerations

**iOS**
Prefer using a split view in a regular — not a compact — environment. A split view needs horizontal space in which to display multiple panes. In a compact environment, such as iPhone in portrait orientation, it's difficult to display multiple panes without wrapping or truncating the content, making it less legible and harder to interact with.

**iPadOS**
In iPadOS, a split view can include either two vertical panes, like Mail, or three vertical panes, like Keynote.

Account for narrow, compact, and intermediate window widths. Since iPad windows are fluidly resizable, it's important to consider the design of a split view layout at multiple widths. In particular, ensure that it's possible to navigate between the various panes in a logical way. For guidance, see Layout. For developer guidance, see NavigationSplitView and UISplitViewController.

**macOS**
In macOS, you can arrange the panes of a split view vertically, horizontally, or both. A split view includes dividers between panes that can support dragging to resize them. For developer guidance, see VSplitView and HSplitView.

Set reasonable defaults for minimum and maximum pane sizes. If people can resize the panes in your app's split view, make sure to use sizes that keep the divider visible. If a pane gets too small, the divider can seem to disappear, becoming difficult to use.

Consider letting people hide a pane when it makes sense. If your app includes an editing area, for example, consider letting people hide other panes to reduce distractions or allow more room for editing — in Keynote, people can hide the navigator and presenter notes panes when they want to edit slide content.

Provide multiple ways to reveal hidden panes. For example, you might provide a toolbar button or a menu command — including a keyboard shortcut — that people can use to restore a hidden pane.

Prefer the thin divider style. The thin divider measures one point in width, giving you maximum space for content while remaining easy for people to use. Avoid using thicker divider styles unless you have a specific need. For example, if both sides of a divider present table rows that use strong linear elements that might make a thin divider hard to distinguish, it might work to use a thicker divider. For developer guidance, see NSSplitView.DividerStyle.

**tvOS**
In tvOS, a split view can work well to help people filter content. When people choose a filter category in the primary pane, your app can display the results in the secondary pane.

Choose a split view layout that keeps the panes looking balanced. By default, a split view devotes a third of the screen width to the primary pane and two-thirds to the secondary pane, but you can also specify a half-and-half layout.

Display a single title above a split view, helping people understand the content as a whole. People already know how to use a split view to navigate and filter content; they don't need titles that describe what each pane contains.

Choose the title's alignment based on the type of content the secondary pane contains. Specifically, when the secondary pane contains a content collection, consider centering the title in the window. In contrast, if the secondary pane contains a single main view of important content, consider placing the title above the primary view to give the content more room.

**visionOS**
To display supplementary information, prefer a split view instead of a new window. A split view gives people convenient access to more information without leaving the current context, whereas a new window may confuse people who are trying to navigate or reposition content. Opening more windows also requires you to carefully manage the relationship between views in your app or game. If you need to request a small amount of information or present a simple task that someone must complete before returning to their main task, use a sheet.

**watchOS**
In watchOS, the split view displays either the list view or a detail view as a full-screen view.

Automatically display the most relevant detail view. When your app launches, show people the most pertinent information. For example, display information relevant to their location, the time, or their recent actions.

If your app displays multiple detail pages, place the detail views in a vertical tab view. People can then use the Digital Crown to scroll between the detail view's tabs. watchOS also displays a page indicator next to the Digital Crown, indicating the number of tabs and the currently selected tab.

#### Resources

**Related**
- Sidebars
- Tab bars
- Layout

**Developer documentation**
- NavigationSplitView — SwiftUI
- UISplitViewController — UIKit
- NSSplitViewController — AppKit

**Videos**
- Make your UIKit app more flexible

#### Change log

| Date | Changes |
|---|---|
| June 9, 2025 | Added iOS and iPadOS platform considerations. |
| December 5, 2023 | Added guidance for split views in visionOS. |
| June 5, 2023 | Added guidance for split views in watchOS. |

---

### Tab views
**Path:** Components › Layout and organization
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/tab-views

#### Hero image
![Tab views](../images/components-tab-view-intro@2x.png)
*A stylized representation of a view with three labeled tabs, the first of which is selected. The image is tinted red to subtly reflect the red in the original six-color Apple logo.*

#### Summary
A tab view presents multiple mutually exclusive panes of content in the same area, which people can switch between using a tabbed control.

#### Best practices

Use a tab view to present closely related areas of content. The appearance of a tab view provides a strong visual indication of enclosure. People expect each tab to display content that is in some way similar or related to the content in the other tabs.

Make sure the controls within a pane affect content only in the same pane. Panes are mutually exclusive, so ensure they're fully self-contained.

Provide a label for each tab that describes the contents of its pane. A good label helps people predict the contents of a pane before clicking or tapping its tab. In general, use nouns or short noun phrases for tab labels. A verb or short verb phrase may make sense in some contexts. Use title-style capitalization for tab labels.

Avoid using a pop-up button to switch between tabs. A tabbed control is efficient because it requires a single click or tap to make a selection, whereas a pop-up button requires two. A tabbed control also presents all choices onscreen at the same time, whereas people must click a pop-up button to see its choices. Note that a pop-up button can be a reasonable alternative in cases where there are too many panes of content to reasonably display with tabs.

Avoid providing more than six tabs in a tab view. Having more than six tabs can be overwhelming and create layout issues. If you need to present six or more tabs, consider another way to implement the interface. For example, you could instead present each tab as a view option in a pop-up button menu.

For developer guidance, see NSTabView.

#### Anatomy

The tabbed control appears on the top edge of the content area. You can choose to hide the control, which is appropriate for an app that switches between panes programmatically.

When you hide the tabbed control, the content area can be borderless, bezeled, or bordered with a line. A borderless view can be solid or transparent.

In general, inset a tab view by leaving a margin of window-body area on all sides of a tab view. This layout looks clean and leaves room for additional controls that aren't directly related to the contents of the tab view. You can extend a tab view to meet the window edges, but this layout is unusual.

#### Platform considerations

Not supported in iOS, iPadOS, tvOS, or visionOS.

**iOS, iPadOS**
For similar functionality, consider using a segmented control instead.

**watchOS**
watchOS displays tab views using page controls. For developer guidance, see TabView and verticalPage.

#### Resources

**Related**
- Tab bars
- Segmented controls

**Developer documentation**
- TabView — SwiftUI
- NSTabView — AppKit

#### Change log

| Date | Changes |
|---|---|
| June 5, 2023 | Added guidance for using tab views in watchOS. |
