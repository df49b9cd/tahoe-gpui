## Components › Content

The Content subcategory covers components that display media and structured text: Charts for data visualization, Image views for photos and graphics, Text views for editable and scrollable text, and Web views for rendering web content inline.

### Section map

| Page | Canonical URL |
|---|---|
| Charts | https://developer.apple.com/design/human-interface-guidelines/charts |
| Image views | https://developer.apple.com/design/human-interface-guidelines/image-views |
| Text views | https://developer.apple.com/design/human-interface-guidelines/text-views |
| Web views | https://developer.apple.com/design/human-interface-guidelines/web-views |

### Detailed pages

---

### Charts
**Path:** Components › Content
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/charts

#### Hero image
![Charts](../images/components-charts-intro@2x.png)
*A stylized representation of a bar chart. The image is tinted red to subtly reflect the red in the original six-color Apple logo.*

#### Summary
Charts help people understand data. A well-designed chart uses visual elements to present data in a way that's far easier to comprehend than a table of numbers.

You can use a chart to help people visualize the relationship between data values. For example, a chart might help people understand how values compare to each other or to a fixed scale, how values change over time, or how a set of data is distributed.

#### Best practices

Use charts to convey meaning that's difficult to convey with numbers alone. It can make sense to include a chart even when you also display the underlying data, because the chart can reveal patterns and relationships that are hard to see in a table.

Choose a chart type that best represents the relationships in your data and supports the tasks you want to enable. For example, a bar chart is great for comparing amounts, whereas a line chart is great for showing trends over time.

Aim to make charts accessible to everyone. Many people use assistive technologies — like VoiceOver, Dynamic Type, or Switch Control — on their devices every day. When you design a chart, make sure your design accommodates all the ways people might interact with it. For example, make sure that information conveyed through color alone is also available in other ways.

Avoid using more than a few colors in a chart. Charts that use too many colors can be confusing and hard for people with color vision deficiency to use.

Design custom marks carefully. Marks are the graphical representations of data in a chart, such as bars or lines. When you use a custom mark in place of a standard one, make sure it's clearly associated with the underlying data value it represents.

#### Chart types

**Bar charts**
A bar chart displays data as horizontal or vertical rectangular bars. Bar charts are good for comparing values across categories or showing change over time when you have a small number of time periods.

**Line charts**
A line chart displays data as points connected by lines. Line charts are good for showing trends over time or comparing trends across multiple data series.

**Area charts**
An area chart is like a line chart, except that the region between the line and the baseline is filled with color. Area charts are good for showing the overall trend of a value over time, and can show part-to-whole relationships.

**Point charts**
A point chart (also known as a scatter plot) displays data as a series of points. Point charts are good for showing the relationship between two different variables or showing the distribution of data.

**Range charts**
A range chart displays a range of values for each data point. Range charts are good for showing confidence intervals, weather forecasts, and other range-based data.

**Rule marks**
A rule mark displays a single horizontal or vertical line. Rule marks are good for indicating a threshold or target value.

#### Axes

Axes help people understand the scale and range of values in a chart. When you include axes, use clear and concise labels.

**Gridlines**
Gridlines extend across the plot area and help people estimate the value of individual marks. Use gridlines sparingly to avoid cluttering the chart.

#### Accessibility

Make sure important information is not conveyed by color alone. Consider providing additional cues, such as different shapes for marks or patterns for fills.

**VoiceOver**
Make sure that VoiceOver can navigate and describe the data in your chart. Provide descriptive audio labels for chart elements and use the Swift Charts accessibility APIs to expose data values.

#### Platform considerations

**iOS, iPadOS**
Swift Charts is available in iOS 16 and later.

**macOS**
Swift Charts is available in macOS 13 and later.

**tvOS**
Swift Charts is available in tvOS 16 and later.

**watchOS**
Swift Charts is available in watchOS 9 and later.

#### Resources

**Related**
- Accessibility
- Color
- Layout

**Developer documentation**
- Chart — Swift Charts

**Videos**
- Hello Swift Charts
- Swift Charts: Raise the bar

---

### Image views
**Path:** Components › Content
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/image-views

#### Hero image
![Image views](../images/components-image-view-intro@2x.png)
*A stylized representation of an image view displaying a landscape photograph. The image is tinted red to subtly reflect the red in the original six-color Apple logo.*

#### Summary
An image view displays a single image or an animated sequence of images over a transparent or opaque background.

Within an image view, you can stretch, scale, size to fit, or pin the image to a specific location. Image views are typically not interactive, although you can add a gesture recognizer to an image view to let people interact with it.

#### Best practices

Use the highest-resolution images you can. If your app uses lower-resolution images, they can appear blurry, especially in full-screen contexts.

Keep in mind that the system automatically draws images in the correct orientation. Regardless of the device orientation or display size, the system renders images correctly within their image views.

Avoid using a standard image view to display images that also need a visible caption. Use a custom component that includes both an image view and a text label if you want to display a caption with an image.

Don't display a border on an image unless doing so helps people understand that the image is a separate component. In general, an image fits more naturally in an interface when it doesn't have a border.

#### Scaling

If your image and image view have different sizes, use scaling behavior to ensure the best appearance.

**Scale to fill**
Scale to fill stretches the image to fill the view. If the image has a different aspect ratio than the view, the image appears cropped.

**Aspect fit**
Aspect fit scales the image to fill one dimension of the view while maintaining the image's aspect ratio. If the image has a different aspect ratio than the view, space appears on either side.

**Aspect fill**
Aspect fill scales the image to fill the view while maintaining the image's aspect ratio. If the image has a different aspect ratio than the view, the image appears cropped.

#### Platform considerations

**iOS, iPadOS**
For developer guidance, see UIImageView.

**macOS**
For developer guidance, see NSImageView.

**tvOS**
Image views can have a layered appearance that creates a sense of depth when people focus on them using the remote or a game controller.

#### Resources

**Related**
- Images
- SF Symbols
- Layout

**Developer documentation**
- Image — SwiftUI
- UIImageView — UIKit
- NSImageView — AppKit

---

### Text views
**Path:** Components › Content
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/text-views

#### Hero image
![Text views](../images/components-text-view-intro@2x.png)
*A stylized representation of a text view showing multiple lines of text. The image is tinted red to subtly reflect the red in the original six-color Apple logo.*

#### Summary
A text view displays multiline, styled text content. Text views can be any height and support scrolling when the content is taller than the view. By default, text within a text view is aligned to the leading edge and uses the system font in black.

If the view is editable, a keyboard appears when people tap or click inside it. A text view is a versatile component for displaying large amounts of text, including long-form text that doesn't fit in a single line.

#### Best practices

Show the appropriate keyboard type. Consider the type of content that someone might enter when you configure a text view. For example, if you know someone might enter a URL or a phone number, you might choose a keyboard type that makes entering that content easier.

Consider using a text field instead when text input is short and single-line. Text views work best for longer-form text, whereas text fields are better for short, single-line input.

**Fonts and styles**
Support Dynamic Type to make text adapt to the user's preferred reading size. Use semantic text styles — like body, headline, or footnote — instead of fixed font sizes. This ensures your app's text scales properly when someone changes their preferred text size in Settings.

Use text colors that have sufficient contrast. Make sure your text is legible against its background by using colors with appropriate contrast ratios.

#### Platform considerations

**iOS, iPadOS**
For developer guidance, see UITextView.

**macOS**
For developer guidance, see NSTextView.

**watchOS**
Text views are read-only in watchOS. For developer guidance, see Text.

#### Resources

**Related**
- Typography
- Color
- Accessibility

**Developer documentation**
- TextEditor — SwiftUI
- UITextView — UIKit
- NSTextView — AppKit

---

### Web views
**Path:** Components › Content
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/web-views

#### Hero image
![Web views](../images/components-web-view-intro@2x.png)
*A stylized representation of a web view displaying a webpage. The image is tinted red to subtly reflect the red in the original six-color Apple logo.*

#### Summary
A web view loads and displays rich web content, such as embedded HTML and websites, directly within your app.

For example, Mail uses a web view to display HTML content in email messages.

#### Best practices

When appropriate, let people navigate within a web view. Web content can contain links to other pages. Unless there's a good reason not to, let people follow links within a web view. However, if the web view is not the primary focus of your app — for example, if it's showing an advertisement — prevent navigation that takes people away from your app's content.

Display a loading indicator when content is loading. Web pages can take time to load, especially when the user has a slow network connection. Help people understand that content is on its way by displaying an activity indicator or progress bar when a page is loading.

Avoid providing a web view that people will use to browse web content generally. If your app needs to let people browse the web generally, use Safari. An in-app browser doesn't include the security, privacy, and functionality of Safari.

Don't use a web view to present custom UI that mimics native UI. If you want to create a custom interface, use native components and SwiftUI, UIKit, or AppKit. Don't create a "fake" native UI using HTML and CSS rendered in a web view.

#### Platform considerations

**iOS, iPadOS**
For developer guidance, see WKWebView.

**macOS**
For developer guidance, see WKWebView.

**tvOS**
Not supported. Consider whether displaying web content within a TV interface makes sense for your use case.

**watchOS**
Not supported.

#### Resources

**Related**
- Safari
- Loading

**Developer documentation**
- WKWebView — WebKit

**Videos**
- What's new in Safari web extensions
