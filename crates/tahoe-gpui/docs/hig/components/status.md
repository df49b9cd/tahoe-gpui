## Components › Status

The Status subcategory covers components that communicate information about the current state of a task, process, or value — helping people understand what is happening without requiring interaction.

| Page | Path | URL |
|---|---|---|
| Activity rings | Components › Status › Activity rings | https://developer.apple.com/design/human-interface-guidelines/activity-rings |
| Gauges | Components › Status › Gauges | https://developer.apple.com/design/human-interface-guidelines/gauges |
| Progress indicators | Components › Status › Progress indicators | https://developer.apple.com/design/human-interface-guidelines/progress-indicators |
| Rating indicators | Components › Status › Rating indicators | https://developer.apple.com/design/human-interface-guidelines/rating-indicators |

---

### Activity rings

**Path:** Components › Status › Activity rings
**URL:** https://developer.apple.com/design/human-interface-guidelines/activity-rings
**Hero image:** `../images/components-activity-ring-intro@2x.png`
*A stylized representation of a set of move, exercise, and stand activity rings denoting progress.*

Activity rings show an individual's daily progress toward Move, Exercise, and Stand goals. In watchOS, the Activity ring element always contains three rings, whose colors and meanings match those the Activity app provides. In iOS, the Activity ring element contains either a single Move ring representing an approximation of activity, or all three rings if an Apple Watch is paired.

---

#### Best practices

Display Activity rings when they're relevant to the purpose of your app. If your app is related to health or fitness, and especially if it contributes information to HealthKit, people generally expect to find Activity rings in your interface. For example, if you structure a workout or health session around the completion of Activity rings, consider displaying the element on a workout metrics screen so that people can track their progress during their session. Similarly, if you provide a summary screen that appears at the conclusion of a workout, you could display Activity rings to help people check on their progress toward their daily goals.

Use Activity rings only to show Move, Exercise, and Stand information. Activity rings are designed to consistently represent progress in these specific areas. Don't replicate or modify Activity rings for other purposes. Never use Activity rings to display other types of data. Never show Move, Exercise, and Stand progress in another ring-like element.

Use Activity rings to show progress for a single person. Never use Activity rings to represent data for more than one person, and make sure it's obvious whose progress you're showing by using a label, a photo, or an avatar.

Always keep the visual appearance of Activity rings the same, regardless of where you display them. Follow these guidelines to provide a consistent experience:

- Never change the colors of the rings; for example, don't use filters or modify opacity.
- Always display Activity rings on a black background.
- Prefer enclosing the rings and background within a circle. To do this, adjust the corner radius of the enclosing view rather than applying a circular mask.
- Ensure that the black background remains visible around the outermost ring. If necessary, add a thin, black stroke around the outer edge of the ring, and avoid including a gradient, shadow, or any other visual effect.
- Always scale the rings appropriately so they don't seem disconnected or out of place.
- When necessary, design the surrounding interface to blend with the rings; never change the rings to blend with the surrounding interface.

To display a label or value that's directly associated with an Activity ring, use the colors that match it. To display the ring-specific labels Move, Exercise, and Stand, or to display a person's current and goal values for each ring, use the following colors, specified as RGB values.

| Move | Exercise | Stand |
|---|---|---|
| R-250, G-17, B-79 | R-166, G-255, B-0 | R-0, G-255, B-246 |

Maintain Activity ring margins. An Activity ring element must include a minimum outer margin of no less than the distance between rings. Never allow other elements to crop, obstruct, or encroach upon this margin or the rings themselves.

Differentiate other ring-like elements from Activity rings. Mixing different ring styles can lead to a visually confusing interface. If you must include other rings, use padding, lines, or labels to separate them from Activity rings. Color and scale can also help provide visual separation.

Don't send notifications that repeat the same information the Activity app sends. The system already delivers Move, Exercise, and Stand progress updates, so it's confusing for people to receive redundant information from your app. Also, don't show an Activity ring element in your app's notifications. It's fine to reference Activity progress in a notification, but do so in a way that's unique to your app and doesn't replicate the same information the system provides.

Don't use Activity rings for decoration. Activity rings provide information to people; they don't just embellish your app's design. Never display Activity rings in labels or background graphics.

Don't use Activity rings for branding. Use Activity rings strictly to display Activity progress in your app. Never use Activity rings in your app's icon or marketing materials.

---

#### Platform considerations

No additional considerations for iPadOS or watchOS. Not supported in macOS, tvOS, or visionOS.

**iOS**

Activity rings are available in iOS with HKActivityRingView. The appearance of the Activity ring element changes automatically depending on whether an Apple Watch is paired:

- With an Apple Watch paired, iOS shows all three Activity rings.
- Without an Apple Watch paired, iOS shows the Move ring only, which represents an approximation of a person's activity based on their steps and workout information from other apps.

Because iOS shows Activity rings whether or not an Apple Watch is paired, activity history can include a combination of both styles. For example, Activity rings in Fitness have three rings when a person exercises with their Apple Watch paired, and only the Move ring when they exercise without their Apple Watch.

---

#### Resources

**Related:** Workouts

**Developer documentation:**
- HKActivityRingView — HealthKit

**Videos:**
- Track workouts with HealthKit on iOS and iPadOS (WWDC25)
- Build a workout app for Apple Watch (WWDC21)
- Build custom workouts with WorkoutKit (WWDC23)

---

#### Change log

| Date | Changes |
|---|---|
| March 29, 2024 | Enhanced guidance for displaying Activity rings and listed specific colors for displaying related content. |
| December 5, 2023 | Added artwork representing Activity rings in iOS. |

---

### Gauges

**Path:** Components › Status › Gauges
**URL:** https://developer.apple.com/design/human-interface-guidelines/gauges
**Hero image:** `../images/components-gauges-intro@2x.png`
*A stylized representation of a circular numeric gauge above a linear percentage gauge.*

A gauge displays a specific numerical value within a range of values. In addition to indicating the current value in a range, a gauge can provide more context about the range itself. For example, a temperature gauge can use text to identify the highest and lowest temperatures in the range and display a spectrum of colors that visually reinforce the changing values.

---

#### Anatomy

A gauge uses a circular or linear path to represent a range of values, mapping the current value to a specific point on the path. A standard gauge displays an indicator that shows the current value's location; a gauge that uses the capacity style displays a fill that stops at the value's location on the path.

Circular and linear gauges in both standard and capacity styles are also available in a variant that's visually similar to watchOS complications. This variant — called accessory — works well in iOS Lock Screen widgets and anywhere you want to echo the appearance of complications.

> Note: In addition to gauges, macOS also supports level indicators, some of which have visual styles that are similar to gauges. For guidance, see the macOS platform considerations below.

---

#### Best practices

Write succinct labels that describe the current value and both endpoints of the range. Although not every gauge style displays all labels, VoiceOver reads the visible labels to help people understand the gauge without seeing the screen.

Consider filling the path with a gradient to help communicate the purpose of the gauge. For example, a temperature gauge might use colors that range from red to blue to represent temperatures that range from hot to cold.

---

#### Platform considerations

No additional considerations for iOS, iPadOS, visionOS, or watchOS. Not supported in tvOS.

**macOS**

In addition to supporting gauges, macOS also defines a level indicator that displays a specific numerical value within a range. You can configure a level indicator to convey capacity, rating, or — rarely — relevance.

The capacity style can depict discrete or continuous values.

**Continuous.** A horizontal translucent track that fills with a solid bar to indicate the current value.

**Discrete.** A horizontal row of separate, equally sized, rectangular segments. The number of segments matches the total capacity, and the segments fill completely — never partially — with color to indicate the current value.

Consider using the continuous style for large ranges. A large value range can make the segments of a discrete capacity indicator too small to be useful.

Consider changing the fill color to inform people about significant parts of the range. By default, the fill color for both capacity indicator styles is green. If it makes sense in your app, you can change the fill color when the current value reaches certain levels, such as very low, very high, or just past the middle. You can change the fill color of the entire indicator or you can use the tiered state to show a sequence of several colors in one indicator.

For guidance using the rating style to help people rank something, see Rating indicators.

Although rarely used, the relevance style can communicate relevancy using a shaded horizontal bar. For example, a relevance indicator might appear in a list of search results, helping people visualize the relevancy of the results when sorting or comparing multiple items.

---

#### Resources

**Related:** Ratings and reviews

**Developer documentation:**
- Gauge — SwiftUI
- NSLevelIndicator — AppKit

---

#### Change log

| Date | Changes |
|---|---|
| September 23, 2022 | New page. |

---

### Progress indicators

**Path:** Components › Status › Progress indicators
**URL:** https://developer.apple.com/design/human-interface-guidelines/progress-indicators
**Hero image:** `../images/components-progress-indicators-intro@2x.png`
*A stylized representation of a spinning indeterminate activity indicator above a progress bar.*

Some progress indicators also give people a way to estimate how long they have to wait for something to complete. All progress indicators are transient, appearing only while an operation is ongoing and disappearing after it completes.

Because the duration of an operation is either known or unknown, there are two types of progress indicators:

- Determinate, for a task with a well-defined duration, such as a file conversion
- Indeterminate, for unquantifiable tasks, such as loading or synchronizing complex data

Both determinate and indeterminate progress indicators can have different appearances depending on the platform. A determinate progress indicator shows the progress of a task by filling a linear or circular track as the task completes. Progress bars include a track that fills from the leading side to the trailing side. Circular progress indicators have a track that fills in a clockwise direction.

An indeterminate progress indicator — also called an activity indicator — uses an animated image to indicate progress. All platforms support a circular image that appears to spin; however, macOS also supports an indeterminate progress bar.

For developer guidance, see ProgressView.

---

#### Best practices

When possible, use a determinate progress indicator. An indeterminate progress indicator shows that a process is occurring, but it doesn't help people estimate how long a task will take. A determinate progress indicator can help people decide whether to do something else while waiting for the task to complete, restart the task at a different time, or abandon the task.

Be as accurate as possible when reporting advancement in a determinate progress indicator. Consider evening out the pace of advancement to help people feel confident about the time needed for the task to complete. Showing 90 percent completion in five seconds and the last 10 percent in 5 minutes can make people wonder if your app is still working and can even feel deceptive.

Keep progress indicators moving so people know something is continuing to happen. People tend to associate a stationary indicator with a stalled process or a frozen app. If a process stalls for some reason, provide feedback that helps people understand the problem and what they can do about it.

When possible, switch a progress bar from indeterminate to determinate. If an indeterminate process reaches a point where you can determine its duration, switch to a determinate progress bar. People generally prefer a determinate progress indicator, because it helps them gauge what's happening and how long it will take.

Don't switch from the circular style to the bar style. Activity indicators (also called spinners) and progress bars are different shapes and sizes, so transitioning between them can disrupt your interface and confuse people.

If it's helpful, display a description that provides additional context for the task. Be accurate and succinct. Avoid vague terms like loading or authenticating because they seldom add value.

Display a progress indicator in a consistent location. Choosing a consistent location for a progress indicator helps people reliably find the status of an operation across platforms or within or between apps.

When it's feasible, let people halt processing. If people can interrupt a process without causing negative side effects, include a Cancel button. If interrupting the process might cause negative side effects — such as losing the downloaded portion of a file — it can be useful to provide a Pause button in addition to a Cancel button.

Let people know when halting a process has a negative consequence. When canceling a process results in lost progress, it's helpful to provide an alert that includes an option to confirm the cancellation or resume the process.

---

#### Platform considerations

No additional considerations for tvOS or visionOS.

**iOS, iPadOS**

**Refresh content controls**

A refresh control lets people immediately reload content, typically in a table view, without waiting for the next automatic content update to occur. A refresh control is a specialized type of activity indicator that's hidden by default, becoming visible when people drag down the view they want to reload. In Mail, for example, people can drag down the list of Inbox messages to check for new messages.

Perform automatic content updates. Although people appreciate being able to do an immediate content refresh, they also expect automatic refreshes to occur periodically. Don't make people responsible for initiating every update. Keep data fresh by updating it regularly.

Supply a short title only if it adds value. Optionally, a refresh control can include a title. In most cases, this is unnecessary, as the animation of the control indicates that content is loading. If you do include a title, don't use it to explain how to perform a refresh. Instead, provide information of value about the content being refreshed. A refresh control in Podcasts, for example, uses a title to tell people when the last podcast update occurred.

For developer guidance, see UIRefreshControl.

**macOS**

In macOS, an indeterminate progress indicator can have a bar or circular appearance. Both versions use an animated image to indicate that the app is performing a task.

Prefer an activity indicator (spinner) to communicate the status of a background operation or when space is constrained. Spinners are small and unobtrusive, so they're useful for asynchronous background tasks, like retrieving messages from a server. Spinners are also good for communicating progress within a small area, such as within a text field or next to a specific control, such as a button.

Avoid labeling a spinning progress indicator. Because a spinner typically appears when people initiate a process, a label is usually unnecessary.

**watchOS**

By default the system displays the progress indicators in white over the scene's background color. You can change the color of the progress indicator by setting its tint color.

---

#### Resources

**Developer documentation:**
- ProgressView — SwiftUI
- UIProgressView — UIKit
- UIActivityIndicatorView — UIKit
- UIRefreshControl — UIKit
- NSProgressIndicator — AppKit

---

#### Change log

| Date | Changes |
|---|---|
| September 12, 2023 | Combined guidance common to all platforms. |
| June 5, 2023 | Updated guidance to reflect changes in watchOS 10. |

---

### Rating indicators

**Path:** Components › Status › Rating indicators
**URL:** https://developer.apple.com/design/human-interface-guidelines/rating-indicators
**Hero image:** `../images/components-rating-indicators-intro@2x.png`
*A stylized representation of a rating indicator denoting a ranking of three out of five stars.*

A rating indicator uses a series of horizontally arranged graphical symbols — by default, stars — to communicate a ranking level. A rating indicator doesn't display partial symbols; it rounds the value to display complete symbols only. Within a rating indicator, symbols are always the same distance apart and don't expand or shrink to fit the component's width.

---

#### Best practices

Make it easy to change rankings. When presenting a list of ranked items, let people adjust the rank of individual items inline without navigating to a separate editing screen.

If you replace the star with a custom symbol, make sure that its purpose is clear. The star is a very recognizable ranking symbol, and people may not associate other symbols with a rating scale.

---

#### Platform considerations

No additional considerations for macOS. Not supported in iOS, iPadOS, tvOS, visionOS, or watchOS.

---

#### Resources

**Related:** Ratings and reviews

**Developer documentation:**
- NSLevelIndicator.Style.rating — AppKit

---

#### Change log

| Date | Changes |
|---|---|
| September 23, 2022 | New page. |
