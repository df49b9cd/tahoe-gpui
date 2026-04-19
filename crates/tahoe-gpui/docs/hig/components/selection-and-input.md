## Components › Selection and input

Controls and views that let people select values or enter data — including color wells, combo boxes, digit entry views, image wells, pickers, segmented controls, sliders, steppers, text fields, toggles, and virtual keyboards.

| Page | Path | Platforms |
|------|------|-----------|
| [Color wells](#color-wells) | Components › Selection and input › Color wells | macOS |
| [Combo boxes](#combo-boxes) | Components › Selection and input › Combo boxes | macOS |
| [Digit entry views](#digit-entry-views) | Components › Selection and input › Digit entry views | tvOS |
| [Image wells](#image-wells) | Components › Selection and input › Image wells | macOS |
| [Pickers](#pickers) | Components › Selection and input › Pickers | iOS, iPadOS, macOS, tvOS, visionOS, watchOS |
| [Segmented controls](#segmented-controls) | Components › Selection and input › Segmented controls | iOS, iPadOS, macOS, tvOS, visionOS |
| [Sliders](#sliders) | Components › Selection and input › Sliders | iOS, iPadOS, macOS, visionOS, watchOS |
| [Steppers](#steppers) | Components › Selection and input › Steppers | iOS, iPadOS, macOS, visionOS |
| [Text fields](#text-fields) | Components › Selection and input › Text fields | iOS, iPadOS, macOS, tvOS, visionOS, watchOS |
| [Toggles](#toggles) | Components › Selection and input › Toggles | iOS, iPadOS, macOS, tvOS, visionOS, watchOS |
| [Virtual keyboards](#virtual-keyboards) | Components › Selection and input › Virtual keyboards | iOS, iPadOS, tvOS, visionOS, watchOS |

---

### Color wells

**Path:** Components › Selection and input › Color wells
**URL:** https://developer.apple.com/design/human-interface-guidelines/color-wells

![Color wells hero](../images/components-color-well-intro@2x.png)

A color well is a button that displays a color and, when activated, opens a color picker that lets people choose a color.

#### Best practices

Use a color well when people need to choose a specific color. Color wells are most appropriate when you need to let people select a custom color rather than from a predefined set. If you only need people to choose from a small set of colors, consider using a row of colored buttons instead.

Consider placing color wells in a toolbar or inspector. Color wells are best used in a context where color selection is a secondary task, not the primary focus of the interface.

#### Platform considerations

Not supported in iOS, iPadOS, tvOS, visionOS, or watchOS.

#### Resources

**Developer documentation**
- NSColorWell — AppKit

---

### Combo boxes

**Path:** Components › Selection and input › Combo boxes
**URL:** https://developer.apple.com/design/human-interface-guidelines/combo-boxes

![Combo boxes hero](../images/components-combobox-intro@2x.png)

A combo box combines a text field with a list of options, giving people the choice between entering a value or choosing from a list.

#### Best practices

Use a combo box when a text field paired with a list of choices makes sense. A combo box is appropriate when there are common choices that people are likely to select, but you also need to allow people to enter values not in the list — like a font size field that provides common sizes but also lets people type any size.

Populate the list with the most useful choices. List only choices that are likely to be valuable, such as common settings or recently used values.

Don't use a combo box if people can only choose from a list. If people can't enter custom values, use a pop-up button or other list-based control instead.

#### Platform considerations

Not supported in iOS, iPadOS, tvOS, visionOS, or watchOS.

#### Resources

**Developer documentation**
- NSComboBox — AppKit

---

### Digit entry views

**Path:** Components › Selection and input › Digit entry views
**URL:** https://developer.apple.com/design/human-interface-guidelines/digit-entry-views

![Digit entry views hero](../images/components-digit-entry-view-intro@2x.png)

A digit entry view presents a row of individual digit fields that people use to enter a numeric code or passcode.

You can add an optional title and prompt above the line of digits.

#### Best practices

**Use secure digit fields.** Secure digit fields display asterisks instead of the entered digit onscreen. Always use a secure digit field when your app asks for sensitive data.

**Clearly state the purpose of the digit entry view.** Use a title and prompt that explains why someone needs to enter digits.

#### Platform considerations

Not supported in iOS, iPadOS, macOS, visionOS, or watchOS.

#### Resources

**Related**
- Virtual keyboards

**Developer documentation**
- TVDigitEntryViewController — TVUIKit

---

### Image wells

**Path:** Components › Selection and input › Image wells
**URL:** https://developer.apple.com/design/human-interface-guidelines/image-wells

![Image wells hero](../images/components-image-well-intro@2x.png)

An image well is an editable version of an image view.

After selecting an image well, people can copy and paste its image or delete it. People can also drag a new image into an image well without selecting it first.

#### Best practices

**Revert to a default image when necessary.** If your image well requires an image, display the default image again if people clear the content of the image well.

**If your image well supports copy and paste, make sure the standard copy and paste menu items are available.** People generally expect to choose these menu items — or use the standard keyboard shortcuts — to interact with an image well. For guidance, see Edit menu.

For related guidance, see Image views.

#### Platform considerations

Not supported in iOS, iPadOS, tvOS, visionOS, or watchOS.

#### Resources

**Related**
- Image views

**Developer documentation**
- NSImageView — AppKit

---

### Pickers

**Path:** Components › Selection and input › Pickers
**URL:** https://developer.apple.com/design/human-interface-guidelines/pickers

![Pickers hero](../images/components-pickers-intro@2x.png)

A picker displays one or more scrollable lists of distinct values that people can choose from.

The system provides several styles of pickers, each of which offers different types of selectable values and has a different appearance. The exact values shown in a picker, and their order, depend on the device language.

Pickers help people enter information by letting them choose single or multipart values. Date pickers specifically offer additional ways to choose values, like selecting a day in a calendar view or entering dates and times using a numeric keypad.

#### Best practices

**Consider using a picker to offer medium-to-long lists of items.** If you need to display a fairly short list of choices, consider using a pull-down button instead of a picker. Although a picker makes it easy to scroll quickly through many items, it may add too much visual weight to a short list of items. On the other hand, if you need to present a very large set of items, consider using a list or table. Lists and tables can adjust in height, and tables can include an index, which makes it much faster to target a section of the list.

**Use predictable and logically ordered values.** Before people interact with a picker, many of its values can be hidden. It's best when people can predict what the hidden values are, such as with an alphabetized list of countries, so they can move through the items quickly.

**Avoid switching views to show a picker.** A picker works well when displayed in context, below or in proximity to the field people are editing. A picker typically appears at the bottom of a window or in a popover.

**Consider providing less granularity when specifying minutes in a date picker.** By default, a minute list includes 60 values (0 to 59). You can optionally increase the minute interval as long as it divides evenly into 60. For example, you might want quarter-hour intervals (0, 15, 30, and 45).

#### Platform considerations

No additional considerations for visionOS.

**iOS, iPadOS**

A date picker is an efficient interface for selecting a specific date, time, or both, using touch, a keyboard, or a pointing device. You can display a date picker in one of the following styles:

- Compact — A button that displays editable date and time content in a modal view.
- Inline — For time only, a button that displays wheels of values; for dates and times, an inline calendar view.
- Wheels — A set of scrolling wheels that also supports data entry through built-in or external keyboards.
- Automatic — A system-determined style based on the current platform and date picker mode.

A date picker has four modes, each of which presents a different set of selectable values.

- Date — Displays months, days of the month, and years.
- Time — Displays hours, minutes, and (optionally) an AM/PM designation.
- Date and time — Displays dates, hours, minutes, and (optionally) an AM/PM designation.
- Countdown timer — Displays hours and minutes, up to a maximum of 23 hours and 59 minutes. This mode isn't available in the inline or compact styles.

The exact values shown in a date picker, and their order, depend on the device location.

**Use a compact date picker when space is constrained.** The compact style displays a button that shows the current value in your app's accent color. When people tap the button, the date picker opens a modal view, providing access to a familiar calendar-style editor and time picker. Within the modal view, people can make multiple edits to dates and times before tapping outside the view to confirm their choices.

**macOS**

**Choose a date picker style that suits your app.** There are two styles of date pickers in macOS: textual and graphical. The textual style is useful when you're working with limited space and you expect people to make specific date and time selections. The graphical style is useful when you want to give people the option of browsing through days in a calendar or selecting a range of dates, or when the look of a clock face is appropriate for your app.

For developer guidance, see NSDatePicker.

**tvOS**

Pickers are available in tvOS with SwiftUI. For developer guidance, see Picker.

**watchOS**

Pickers display lists of items that people navigate using the Digital Crown, which helps people manage selections in a precise and engaging way.

A picker can display a list of items using the wheels style. watchOS can also display date and time pickers using the wheels style. For developer guidance, see Picker and DatePicker.

You can configure a picker to display an outline, caption, and scrolling indicator.

For longer lists, the navigation link displays the picker as a button. When someone taps the button, the system shows the list of options. The person can also scrub through the options using the Digital Crown without tapping the button. For developer guidance, see navigationLink.

#### Resources

**Related**
- Pull-down buttons
- Lists and tables

**Developer documentation**
- Picker — SwiftUI
- UIDatePicker — UIKit
- UIPickerView — UIKit
- NSDatePicker — AppKit

#### Change log

| Date | Changes |
|------|---------|
| June 5, 2023 | Updated guidance for using pickers in watchOS. |

---

### Segmented controls

**Path:** Components › Selection and input › Segmented controls
**URL:** https://developer.apple.com/design/human-interface-guidelines/segmented-controls

![Segmented controls hero](../images/components-segmented-control-intro@2x.png)

A segmented control is a linear set of two or more segments, each of which functions as a button.

Within a segmented control, all segments are usually equal in width. Like buttons, segments can contain text or images. Segments can also have text labels beneath them (or beneath the control as a whole).

A segmented control offers a single choice from among a set of options, or in macOS, either a single choice or multiple choices. For example, in macOS Keynote people can select only one segment in the alignment options control to align selected text. In contrast, people can choose multiple segments in the font attributes control to combine styles like bold, italics, and underline. The toolbar of a Keynote window also uses a segmented control to let people show and hide various editing panes within the main window area.

In addition to representing the state of a single or multiple-choice selection, a segmented control can function as a set of buttons that perform actions without showing a selection state. For example, the Reply, Reply all, and Forward buttons in macOS Mail. For developer guidance, see isMomentary and NSSegmentedControl.SwitchTracking.momentary.

#### Best practices

**Use a segmented control to provide closely related choices that affect an object, state, or view.** For example, a segmented control in an inspector could let people choose one or more attributes to apply to a selection, or a segmented control in a toolbar could offer a set of actions to perform on the current view.

In the iOS Health app, a segmented control provides a choice of time ranges for the activity graphs to display.

**Consider a segmented control when it's important to group functions together, or to clearly show their selection state.** Unlike other button styles, segmented controls preserve their grouping regardless of the view size or where they appear. This grouping can also help people understand at a glance which controls are currently selected.

**Keep control types consistent within a single segmented control.** Don't assign actions to segments in a control that otherwise represents selection state, and don't show a selection state for segments in a control that otherwise performs actions.

**Limit the number of segments in a control.** Too many segments can be hard to parse and time-consuming to navigate. Aim for no more than about five to seven segments in a wide interface and no more than about five segments on iPhone.

**In general, keep segment size consistent.** When all segments have equal width, a segmented control feels balanced. To the extent possible, it's best to keep icon and title widths consistent too.

#### Content

**Prefer using either text or images — not a mix of both — in a single segmented control.** Although individual segments can contain text labels or images, mixing the two in a single control can lead to a disconnected and confusing interface.

**As much as possible, use content with a similar size in each segment.** Because all segments typically have equal width, it doesn't look good if content fills some segments but not others.

**Use nouns or noun phrases for segment labels.** Write text that describes each segment and uses title-style capitalization. A segmented control that displays text labels doesn't need introductory text.

#### Platform considerations

Not supported in watchOS.

**iOS, iPadOS**

**Consider a segmented control to switch between closely related subviews.** A segmented control can be useful as a way to quickly switch between related subviews. For example, the segmented control in Calendar's New Event sheet switches between the subviews for creating a new event and a new reminder. For switching between completely separate sections of an app, use a tab bar instead.

**macOS**

**Consider using introductory text to clarify the purpose of a segmented control.** When the control uses symbols or interface icons, you could also add a label below each segment to clarify its meaning. If your app includes tooltips, provide one for each segment in a segmented control.

**Use a tab view in the main window area — instead of a segmented control — for view switching.** A tab view supports efficient view switching and is similar in appearance to a box combined with a segmented control. Consider using a segmented control to help people switch views in a toolbar or inspector pane.

**Consider supporting spring loading.** On a Mac equipped with a Magic Trackpad, spring loading lets people activate a segment by dragging selected items over it and force clicking without dropping the selected items. People can also continue dragging the items after a segment activates.

**tvOS**

**Consider using a split view instead of a segmented control on screens that perform content filtering.** People generally find it easy to navigate back and forth between content and filtering options using a split view. Depending on its placement, a segmented control may not be as easy to access.

**Avoid putting other focusable elements close to segmented controls.** Segments become selected when focus moves to them, not when people click them. Carefully consider where you position a segmented control relative to other interface elements. If other focusable elements are too close, people might accidentally focus on them when attempting to switch between segments.

**visionOS**

When people look at a segmented control that uses icons, the system displays a tooltip that contains the descriptive text you supply.

#### Resources

**Related**
- Split views

**Developer documentation**
- segmented — SwiftUI
- UISegmentedControl — UIKit
- NSSegmentedControl — AppKit

#### Change log

| Date | Changes |
|------|---------|
| June 21, 2023 | Updated to include guidance for visionOS. |

---

### Sliders

**Path:** Components › Selection and input › Sliders
**URL:** https://developer.apple.com/design/human-interface-guidelines/sliders

![Sliders hero](../images/components-slider-intro@2x.png)

A slider is a horizontal track with a control, called a thumb, that people can adjust between a minimum and maximum value.

As a slider's value changes, the portion of track between the minimum value and the thumb fills with color. A slider can optionally display left and right icons that illustrate the meaning of the minimum and maximum values.

#### Best practices

**Customize a slider's appearance if it adds value.** You can adjust a slider's appearance — including track color, thumb image and tint color, and left and right icons — to blend with your app's design and communicate intent. A slider that adjusts image size, for example, could show a small image icon on the left and a large image icon on the right.

**Use familiar slider directions.** People expect the minimum and maximum sides of sliders to be consistent in all apps, with minimum values on the leading side and maximum values on the trailing side (for horizontal sliders) and minimum values at the bottom and maximum values at the top (for vertical sliders). For example, people expect to be able to move a horizontal slider that represents a percentage from 0 percent on the leading side to 100 percent on the trailing side.

**Consider supplementing a slider with a corresponding text field and stepper.** Especially when a slider represents a wide range of values, people may appreciate seeing the exact slider value and having the ability to enter a specific value in a text field. Adding a stepper provides a convenient way for people to increment in whole values. For related guidance, see Text fields and Steppers.

#### Platform considerations

Not supported in tvOS.

**iOS, iPadOS**

**Don't use a slider to adjust audio volume.** If you need to provide volume control in your app, use a volume view, which is customizable and includes a volume-level slider and a control for changing the active audio output device. For guidance, see Playing audio.

**macOS**

Sliders in macOS can also include tick marks, making it easier for people to pinpoint a specific value within the range.

In a linear slider either with or without tick marks, the thumb is a narrow lozenge shape, and the portion of track between the minimum value and the thumb is filled with color. A linear slider often includes supplementary icons that illustrate the meaning of the minimum and maximum values.

In a circular slider, the thumb appears as a small circle. Tick marks, when present, appear as evenly spaced dots around the circumference of the slider.

**Consider giving live feedback as the value of a slider changes.** Live feedback shows people results in real time. For example, your Dock icons are dynamically scaled when adjusting the Size slider in Dock settings.

**Choose a slider style that matches peoples' expectations.** A horizontal slider is ideal when moving between a fixed starting and ending point. For example, a graphics app might offer a horizontal slider for setting the opacity level of an object between 0 and 100 percent. Use circular sliders when values repeat or continue indefinitely. For example, a graphics app might use a circular slider to adjust the rotation of an object between 0 and 360 degrees. An animation app might use a circular slider to adjust how many times an object spins when animated — four complete rotations equals four spins, or 1440 degrees of rotation.

**Consider using a label to introduce a slider.** Labels generally use sentence-style capitalization and end with a colon. For guidance, see Labels.

**Use tick marks to increase clarity and accuracy.** Tick marks help people understand the scale of measurements and make it easier to locate specific values.

**Consider adding labels to tick marks for even greater clarity.** Labels can be numbers or words, depending on the slider's values. It's unnecessary to label every tick mark unless doing so is needed to reduce confusion. In many cases, labeling only the minimum and maximum values is sufficient. When the values of the slider are nonlinear, like in the Energy Saver settings pane, periodic labels provide context. It's also a good idea to provide a tooltip that displays the value of the thumb when people hold their pointer over it.

**visionOS**

**Prefer horizontal sliders.** It's generally easier for people to gesture from side to side than up and down.

**watchOS**

A slider is a horizontal track — appearing as a set of discrete steps or as a continuous bar — that represents a finite range of values. People can tap buttons on the sides of the slider to increase or decrease its value by a predefined amount.

**If necessary, create custom glyphs to communicate what the slider does.** The system displays plus and minus signs by default.

#### Resources

**Related**
- Steppers
- Pickers

**Developer documentation**
- Slider — SwiftUI
- UISlider — UIKit
- NSSlider — AppKit

#### Change log

| Date | Changes |
|------|---------|
| June 21, 2023 | Updated to include guidance for visionOS. |

---

### Steppers

**Path:** Components › Selection and input › Steppers
**URL:** https://developer.apple.com/design/human-interface-guidelines/steppers

![Steppers hero](../images/components-stepper-intro@2x.png)

A stepper is a two-segment control that people use to increase or decrease an incremental value.

A stepper sits next to a field that displays its current value, because the stepper itself doesn't display a value.

#### Best practices

**Make the value that a stepper affects obvious.** A stepper itself doesn't display any values, so make sure people know which value they're changing when they use a stepper.

**Consider pairing a stepper with a text field when large value changes are likely.** Steppers work well by themselves for making small changes that require a few taps or clicks. By contrast, people appreciate the option to use a field to enter specific values, especially when the values they use can vary widely. On a printing screen, for example, it can help to have both a stepper and a text field to set the number of copies.

#### Platform considerations

No additional considerations for iOS, iPadOS, or visionOS. Not supported in watchOS or tvOS.

**macOS**

**For large value ranges, consider supporting Shift-click to change the value quickly.** If your app benefits from larger changes in a stepper's value, it can be useful to let people Shift-click the stepper to change the value by more than the default increment (by 10 times the default, for example).

#### Resources

**Related**
- Pickers
- Text fields

**Developer documentation**
- UIStepper — UIKit
- NSStepper — AppKit

---

### Text fields

**Path:** Components › Selection and input › Text fields
**URL:** https://developer.apple.com/design/human-interface-guidelines/text-fields

![Text fields hero](../images/components-text-field-intro@2x.png)

A text field is a rectangular area in which people enter or edit small, specific pieces of text.

#### Best practices

**Use a text field to request a small amount of information, such as a name or an email address.** To let people input larger amounts of text, use a text view instead.

**Show a hint in a text field to help communicate its purpose.** A text field can contain placeholder text — such as "Email" or "Password" — when there's no other text in the field. Because placeholder text disappears when people start typing, it can also be useful to include a separate label describing the field to remind people of its purpose.

**Use secure text fields to hide private data.** Always use a secure text field when your app asks for sensitive data, such as a password. For developer guidance, see SecureField.

**To the extent possible, match the size of a text field to the quantity of anticipated text.** The size of a text field helps people visually gauge the amount of information to provide.

**Evenly space multiple text fields.** If your layout includes multiple text fields, leave enough space between them so people can easily see which input field belongs with each introductory label. Stack multiple text fields vertically when possible, and use consistent widths to create a more organized layout. For example, the first and last name fields on an address form might be one width, while the address and city fields might be a different width.

**Ensure that tabbing between multiple fields flows as people expect.** When tabbing between fields, move focus in a logical sequence. The system attempts to achieve this result automatically, so you won't need to customize this too often.

**Validate fields when it makes sense.** For example, if the only legitimate value for a field is a string of digits, your app needs to alert people if they've entered characters other than digits. The appropriate time to check the data depends on the context: when entering an email address, it's best to validate when people switch to another field; when creating a user name or password, validation needs to happen before people switch to another field.

**Use a number formatter to help with numeric data.** A number formatter automatically configures the text field to accept only numeric values. It can also display the value in a specific way, such as with a certain number of decimal places, as a percentage, or as currency. Don't assume the actual presentation of data, however, as formatting can vary significantly based on people's locale.

**Adjust line breaks according to the needs of the field.** By default, the system clips any text extending beyond the bounds of a text field. Alternatively, you can set up a text field to wrap text to a new line at the character or word level, or to truncate (indicated by an ellipsis) at the beginning, middle, or end.

**Consider using an expansion tooltip to show the full version of clipped or truncated text.** An expansion tooltip behaves like a regular tooltip and appears when someone places the pointer over the field.

**In iOS, iPadOS, tvOS, and visionOS apps, show the appropriate keyboard type.** Several different keyboard types are available, each designed to facilitate a different type of input, such as numbers or URLs. To streamline data entry, display the keyboard that's appropriate for the type of content people are entering. For guidance, see Virtual keyboards.

**Minimize text entry in your tvOS and watchOS apps.** Entering long passages of text or filling out numerous text fields is time-consuming on Apple TV and Apple Watch. Minimize text input and consider gathering information more efficiently, such as with buttons.

#### Platform considerations

No additional considerations for tvOS or visionOS.

**iOS, iPadOS**

**Display a Clear button in the trailing end of a text field to help people erase their input.** When this element is present, people can tap it to clear the text field's contents, without having to keep tapping the Delete key.

**Use images and buttons to provide clarity and functionality in text fields.** You can display custom images in both ends of a text field, or you can add a system-provided button, such as the Bookmarks button. In general, use the leading end of a text field to indicate a field's purpose and the trailing end to offer additional features, such as bookmarking.

**macOS**

**Consider using a combo box if you need to pair text input with a list of choices.** For related guidance, see Combo boxes.

**watchOS**

**Present a text field only when necessary.** Whenever possible, prefer displaying a list of options rather than requiring text entry.

#### Resources

**Related**
- Text views
- Combo boxes
- Entering data

**Developer documentation**
- TextField — SwiftUI
- SecureField — SwiftUI
- UITextField — UIKit
- NSTextField — AppKit

#### Change log

| Date | Changes |
|------|---------|
| June 5, 2023 | Updated guidance to reflect changes in watchOS 10. |

---

### Toggles

**Path:** Components › Selection and input › Toggles
**URL:** https://developer.apple.com/design/human-interface-guidelines/toggles

![Toggles hero](../images/components-toggles-intro@2x.png)

A toggle lets people choose between a pair of opposing states, like on and off, using a different appearance to indicate each state.

A toggle can have various styles, such as switch and checkbox, and different platforms can use these styles in different ways. For guidance, see Platform considerations.

In addition to toggles, all platforms also support buttons that behave like toggles by using a different appearance for each state. For developer guidance, see ToggleStyle.

#### Best practices

**Use a toggle to help people choose between two opposing values that affect the state of content or a view.** A toggle always lets people manage the state of something, so if you need to support other types of actions — such as choosing from a list of items — use a different component, like a pop-up button.

**Clearly identify the setting, view, or content the toggle affects.** In general, the surrounding context provides enough information for people to understand what they're turning on or off. In some cases, often in macOS apps, you can also supply a label to describe the state the toggle controls. If you use a button that behaves like a toggle, you generally use an interface icon that communicates its purpose, and you update its appearance — typically by changing the background — based on the current state.

**Make sure the visual differences in a toggle's state are obvious.** For example, you might add or remove a color fill, show or hide the background shape, or change the inner details you display — like a checkmark or dot — to show that a toggle is on or off. Avoid relying solely on different colors to communicate state, because not everyone can perceive the differences.

#### Platform considerations

No additional considerations for tvOS, visionOS, or watchOS.

**iOS, iPadOS**

**Use the switch toggle style only in a list row.** You don't need to supply a label in this situation because the content in the row provides the context for the state the switch controls.

**Change the default color of a switch only if necessary.** The default green color tends to work well in most cases, but you might want to use your app's accent color instead. Be sure to use a color that provides enough contrast with the uncolored appearance to be perceptible.

**Outside of a list, use a button that behaves like a toggle, not a switch.** For example, the Phone app uses a toggle on the filter button to let users filter their recent calls. The app adds a blue highlight to indicate when the toggle is active, and removes it when the toggle is inactive.

The Phone app uses a toggle to switch between all recent calls and various filter options. When someone chooses a filter, the toggle appears with a custom background drawn behind the symbol.

When someone returns to the main Recents view, the toggle appears without anything behind the symbol.

**Avoid supplying a label that explains the button's purpose.** The interface icon you create — combined with the alternative background appearances you supply — help people understand what the button does. For developer guidance, see changesSelectionAsPrimaryAction.

**macOS**

In addition to the switch toggle style, macOS supports the checkbox style and also defines radio buttons that can provide similar behaviors.

**Use switches, checkboxes, and radio buttons in the window body, not the window frame.** In particular, avoid using these components in a toolbar or status bar.

**Switches**

**Prefer a switch for settings that you want to emphasize.** A switch has more visual weight than a checkbox, so it looks better when it controls more functionality than a checkbox typically does. For example, you might use a switch to let people turn on or off a group of settings, instead of just one setting. For developer guidance, see switch.

**Within a grouped form, consider using a mini switch to control the setting in a single row.** The height of a mini switch is similar to the height of buttons and other controls, resulting in rows that have a consistent height. If you need to present a hierarchy of settings within a grouped form, you can use a regular switch for the primary setting and mini switches for the subordinate settings. For developer guidance, see GroupedFormStyle and ControlSize.

**In general, don't replace a checkbox with a switch.** If you're already using a checkbox in your interface, it's probably best to keep using it.

**Checkboxes**

A checkbox is a small, square button that's empty when the button is off, contains a checkmark when the button is on, and can contain a dash when the button's state is mixed. Typically, a checkbox includes a title on its trailing side. In an editable checklist, a checkbox can appear without a title or any additional content.

**Use a checkbox instead of a switch if you need to present a hierarchy of settings.** The visual style of checkboxes helps them align well and communicate grouping. By using alignment — generally along the leading edge of the checkboxes — and indentation, you can show dependencies, such as when the state of a checkbox governs the state of subordinate checkboxes.

**Consider using radio buttons if you need to present a set of more than two mutually exclusive options.** When people need to choose from options in addition to just "on" or "off," using multiple radio buttons can help you clarify each option with a unique label.

**Consider using a label to introduce a group of checkboxes if their relationship isn't clear.** Describe the set of options, and align the label's baseline with the first checkbox in the group.

**Accurately reflect a checkbox's state in its appearance.** A checkbox's state can be on, off, or mixed. If you use a checkbox to globally turn on and off multiple subordinate checkboxes, show a mixed state when the subordinate checkboxes have different states. For example, you might need to present a text-style setting that turns all styles on or off, but also lets people choose a subset of individual style settings like bold, italic, or underline. For developer guidance, see allowsMixedState.

**Radio buttons**

A radio button is a small, circular button followed by a label. Typically displayed in groups of two to five, radio buttons present a set of mutually exclusive choices.

A radio button's state is either selected (a filled circle) or deselected (an empty circle). Although a radio button can also display a mixed state (indicated by a dash), this state is rarely useful because you can communicate multiple states by using additional radio buttons. If you need to show that a setting or item has a mixed state, consider using a checkbox instead.

**Prefer a set of radio buttons to present mutually exclusive options.** If you need to let people choose multiple options in a set, use checkboxes instead.

**Avoid listing too many radio buttons in a set.** A long list of radio buttons takes up a lot of space in the interface and can be overwhelming. If you need to present more than about five options, consider using a component like a pop-up button instead.

**To present a single setting that can be on or off, prefer a checkbox.** Although a single radio button can also turn something on or off, the presence or absence of the checkmark in a checkbox can make the current state easier to understand at a glance. In rare cases where a single checkbox doesn't clearly communicate the opposing states, you can use a pair of radio buttons, each with a label that specifies the state it controls.

**Use consistent spacing when you display radio buttons horizontally.** Measure the space needed to accommodate the longest button label, and use that measurement consistently.

#### Resources

**Related**
- Layout

**Developer documentation**
- Toggle — SwiftUI
- UISwitch — UIKit
- NSButton.ButtonType.toggle — AppKit
- NSSwitch — AppKit

#### Change log

| Date | Changes |
|------|---------|
| March 29, 2024 | Enhanced guidance for using switches in macOS apps, clarified when a checkbox has a title, and added artwork for radio buttons. |
| September 12, 2023 | Updated artwork. |

---

### Virtual keyboards

**Path:** Components › Selection and input › Virtual keyboards
**URL:** https://developer.apple.com/design/human-interface-guidelines/virtual-keyboards

![Virtual keyboards hero](../images/components-virtual-keyboard-intro@2x.png)

On devices without physical keyboards, the system offers various types of virtual keyboards people can use to enter data.

A virtual keyboard can provide a specific set of keys that are optimized for the current task; for example, a keyboard that supports entering email addresses can include the "@" character and a period or even ".com". A virtual keyboard doesn't support keyboard shortcuts.

When it makes sense in your app, you can replace the system-provided keyboard with a custom view that supports app-specific data entry. In iOS, iPadOS, and tvOS, you can also create an app extension that offers a custom keyboard people can install and use in place of the standard keyboard.

#### Best practices

**Choose a keyboard that matches the type of content people are editing.** For example, you can help people enter numeric data by providing the numbers and punctuation keyboard. When you specify a semantic meaning for a text input area, the system can automatically provide a keyboard that matches the type of input you expect, potentially using this information to refine the keyboard corrections it offers. For developer guidance, see keyboardType(_:) (SwiftUI), textContentType(_:) (SwiftUI), UIKeyboardType (UIKit), and UITextContentType (UIKit).

**Consider customizing the Return key type if it helps clarify the text-entry experience.** The Return key type is based on the keyboard type you choose, but you can change this if it makes sense in your app. For example, if your app initiates a search, you can use a search Return key type rather than the standard one so the experience is consistent with other places people initiate search. For developer guidance, see submitLabel(_:) (SwiftUI) and UIReturnKeyType (UIKit).

#### Custom input views

In some cases, you can create an input view if you want to provide custom functionality that enhances data-entry tasks in your app. For example, Numbers provides a custom input view for entering numeric values while editing a spreadsheet. A custom input view replaces the system-provided keyboard while people are in your app. For developer guidance, see ToolbarItemPlacement (SwiftUI) and inputViewController (UIKit).

**Make sure your custom input view makes sense in the context of your app.** In addition to making data entry simple and intuitive, you want people to understand the benefits of using your custom input view. Otherwise, they may wonder why they can't regain the system keyboard while in your app.

**Play the standard keyboard sound while people type.** The keyboard sound provides familiar feedback when people tap a key on the system keyboard, so they're likely to expect the same sound when they tap keys in your custom input view. People can turn keyboard sounds off for all keyboard interactions in Settings > Sounds. For developer guidance, see playInputClick() (UIKit).

#### Custom keyboards

In iOS, iPadOS, and tvOS, you can provide a custom keyboard that replaces the system keyboard by creating an app extension. An app extension is code you provide that people can install and use to extend the functionality of a specific area of the system; to learn more, see App extensions.

After people choose your custom keyboard in Settings, they can use it for text entry within any app, except when editing secure text fields and phone number fields. People can choose multiple custom keyboards and switch between them at any time. For developer guidance, see Creating a custom keyboard.

Custom keyboards make sense when you want to expose unique keyboard functionality systemwide, such as a novel way of inputting text or the ability to type in a language the system doesn't support. If you want to provide a custom keyboard for people to use only while they're in your app, consider creating a custom input view instead.

**Provide an obvious and easy way to switch between keyboards.** People know that the Globe key on the standard keyboard — which replaces the dedicated Emoji key when multiple keyboards are available — quickly switches to other keyboards, and they expect a similarly intuitive experience in your keyboard.

**Avoid duplicating system-provided keyboard features.** On some devices, the Emoji/Globe key and Dictation key automatically appear beneath the keyboard, even when people are using custom keyboards. Your app can't affect these keys, and it's likely to be confusing if you repeat them in your keyboard.

**Consider providing a keyboard tutorial in your app.** People are used to the standard keyboard, and learning how to use a new keyboard can take time. You can help make the process easier by providing usage instructions in your app — for example, you might tell people how to choose your keyboard, activate it during text entry, use it, and switch back to the standard keyboard. Avoid displaying help content within the keyboard itself.

#### Platform considerations

Not supported in macOS.

**iOS, iPadOS**

**Use the keyboard layout guide to make the keyboard feel like an integrated part of your interface.** Using the layout guide also helps you keep important parts of your interface visible while the virtual keyboard is onscreen. For developer guidance, see Adjusting your layout with keyboard layout guide.

The keyboard layout guide helps ensure that app UI and the keyboard work well together.

Without the layout guide, the keyboard could make entering text more difficult.

Without the layout guide, the keyboard could make tapping a button more difficult.

**Place custom controls above the keyboard thoughtfully.** Some apps position an input accessory view containing custom controls above the keyboard to offer app-specific functionality related to the data people are working with. For example, Numbers displays controls that help people apply standard or custom calculations to spreadsheet data. If your app offers custom controls that augment the keyboard, make sure they're relevant to the current task. If other views in your app use Liquid Glass, or if your view looks out of place above the keyboard, apply Liquid Glass to the view that contains your controls to maintain consistency. If you use a standard toolbar to contain your controls, it automatically adopts Liquid Glass. Use the keyboard layout guide and standard padding to ensure the system positions your controls as expected within the view. For developer guidance, see ToolbarItemPlacement (SwiftUI), inputAccessoryView (UIKit), and UIKeyboardLayoutGuide (UIKit).

**tvOS**

tvOS displays a linear virtual keyboard when people select a text field using the Siri Remote.

> Note A grid keyboard screen appears when people use devices other than the Siri Remote, and the layout of content automatically adapts to the keyboard.

When people activate a digit entry view, tvOS displays a digit-specific keyboard. For guidance, see Digit entry views.

**visionOS**

In visionOS, the system-provided virtual keyboard supports both direct and indirect gestures and appears in a separate window that people can move where they want. You don't need to account for the location of the keyboard in your layouts.

**watchOS**

On Apple Watch, a text field can show a keyboard if the device screen is large enough. Otherwise, the system lets people use dictation or Scribble to enter information. You can't change the keyboard type in watchOS, but you can set the content type of the text field. The system uses this information to make text entry easier, such as by offering suggestions. For developer guidance, see textContentType(_:) (SwiftUI).

People can also use a nearby paired iPhone to enter text on Apple Watch.

#### Resources

**Related**
- Entering data
- Keyboards
- Layout

**Developer documentation**
- keyboardType(_:) — SwiftUI
- textContentType(_:) — SwiftUI
- UIKeyboardType — UIKit

#### Change log

| Date | Changes |
|------|---------|
| June 9, 2025 | Added guidance for displaying custom controls above the keyboard, and updated to reflect virtual keyboard availability in watchOS. |
| February 2, 2024 | Clarified the virtual keyboard's support for direct and indirect gestures in visionOS. |
| December 5, 2023 | Added artwork for visionOS. |
| June 21, 2023 | Changed page title from Onscreen keyboards and updated to include guidance for visionOS. |
