## Patterns

Use common interaction and flow patterns in ways that feel familiar across Apple platforms.

### Section overview

This section covers 25 interaction and flow patterns defined by the Human Interface Guidelines. Each page documents best practices, platform-specific considerations, and change history for a distinct design pattern — from charting data and collaboration to workouts and settings.

### Section map

| Page | Coverage | Canonical URL |
|------|----------|---------------|
| Charting data | Detailed | https://developer.apple.com/design/human-interface-guidelines/charting-data |
| Collaboration and sharing | Detailed | https://developer.apple.com/design/human-interface-guidelines/collaboration-and-sharing |
| Drag and drop | Detailed | https://developer.apple.com/design/human-interface-guidelines/drag-and-drop |
| Entering data | Detailed | https://developer.apple.com/design/human-interface-guidelines/entering-data |
| Feedback | Detailed | https://developer.apple.com/design/human-interface-guidelines/feedback |
| File management | Detailed | https://developer.apple.com/design/human-interface-guidelines/file-management |
| Going full screen | Detailed | https://developer.apple.com/design/human-interface-guidelines/going-full-screen |
| Launching | Detailed | https://developer.apple.com/design/human-interface-guidelines/launching |
| Live-viewing apps | Detailed | https://developer.apple.com/design/human-interface-guidelines/live-viewing-apps |
| Loading | Detailed | https://developer.apple.com/design/human-interface-guidelines/loading |
| Managing accounts | Detailed | https://developer.apple.com/design/human-interface-guidelines/managing-accounts |
| Managing notifications | Detailed | https://developer.apple.com/design/human-interface-guidelines/managing-notifications |
| Modality | Detailed | https://developer.apple.com/design/human-interface-guidelines/modality |
| Multitasking | Detailed | https://developer.apple.com/design/human-interface-guidelines/multitasking |
| Offering help | Detailed | https://developer.apple.com/design/human-interface-guidelines/offering-help |
| Onboarding | Detailed | https://developer.apple.com/design/human-interface-guidelines/onboarding |
| Playing audio | Detailed | https://developer.apple.com/design/human-interface-guidelines/playing-audio |
| Playing haptics | Detailed | https://developer.apple.com/design/human-interface-guidelines/playing-haptics |
| Playing video | Detailed | https://developer.apple.com/design/human-interface-guidelines/playing-video |
| Printing | Detailed | https://developer.apple.com/design/human-interface-guidelines/printing |
| Ratings and reviews | Detailed | https://developer.apple.com/design/human-interface-guidelines/ratings-and-reviews |
| Searching | Detailed | https://developer.apple.com/design/human-interface-guidelines/searching |
| Settings | Detailed | https://developer.apple.com/design/human-interface-guidelines/settings |
| Undo and redo | Detailed | https://developer.apple.com/design/human-interface-guidelines/undo-and-redo |
| Workouts | Detailed | https://developer.apple.com/design/human-interface-guidelines/workouts |

### Detailed pages

---

### Charting data
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/charting-data

#### Hero image
![Charting data](images/patterns-charting-data-intro@2x.png)
*A sketch of a bar chart, suggesting data representation. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Charts provide efficient ways to communicate complex information without requiring people to read and interpret a lot of text. The graphical nature of charts also gives you additional opportunities to express the personality of your experience and add visual interest to your interface. To learn about the components you use to create a chart, see Charts.

A chart can range from a simple graphic that provides glanceable information to a rich, interactive experience that can form the centerpiece of your app and encourage people to explore the data from various perspectives. Whether simple or complex, you can use charts to help people perform data-driven tasks that are important to them, such as:

- Analyzing trends based on historical or predicted values
- Visualizing the current state of a process, system, or quantity that changes over time
- Evaluating different items — or the same item at different times — by comparing data across multiple categories

Not every collection of data needs to be displayed in a chart. If you simply need to provide data — and you don't need to convey information about it or help people analyze it — consider offering the data in other ways, such as in a list or table that people can scroll, search, and sort.

#### Best practices

Use a chart when you want to highlight important information about a dataset. Charts are visually prominent, so they tend to draw people's attention. Take advantage of this prominence by clearly communicating what people can learn from the data they care about.

Keep a chart simple, letting people choose when they want additional details. Resist the temptation to pack as much data as possible into a chart. Too much data can make a chart visually overwhelming and difficult to use, obscuring the relationships and other information you want to convey. If you have a lot of data to present — or a lot of functionality to provide — consider giving people a way to reveal it gradually. For example, you might let people choose to view different levels of detail or subsets of data to match their interest. To help people learn how to use an interactive chart, you might offer several versions of the chart, each with more functionality than the last.

Make every chart in your app accessible. A chart communicates visually through graphical representations of data and visual descriptions. In addition to the visual descriptions you display, it's crucial to provide both accessibility labels that describe chart values and components, and accessibility elements that help people interact with the chart. For guidance, see Enhancing the accessibility of a chart.

#### Designing effective charts

In general, prefer using common chart types. People tend to be familiar with common chart types — such as bar charts and line charts — so using one of these types in your app can make it more likely that people will already know how to read your chart. For guidance, see Charts.

If you need to create a chart that presents data in a novel way, help people learn how to interpret the chart. For example, when a Watch pairs with iPhone, Activity introduces the Activity rings by animating them individually, showing people how each ring maps to the move, exercise, and stand metrics.

Examine the data from multiple levels or perspectives to find details you can display to enhance the chart. For example, viewing the data from a macro level can help you determine high-level summaries that people might be interested in, like totals or averages. From a mid-level perspective, you might find ways to help people identify useful subsets of the data, whereas examining individual data points might help you find ways to draw people's attention to specific values or items. Displaying information that helps people view the chart from various perspectives can encourage them to engage with it.

Aid comprehension by adding descriptive text to the chart. Descriptive text titles, subtitles, and annotations help emphasize the most important information in a chart and can highlight actionable takeaways. You can also display brief descriptive text that serves as a headline or summary for a chart, helping people grasp essential information at a glance. For example, Weather displays text that summarizes the information people need right now — such as "Chance of light rain in the next hour" — above the scrolling list of hourly forecasts for the next 24 hours. Although a descriptive headline or summary can make a chart more accessible, it doesn't take the place of accessibility labels.

Match the size of a chart to its functionality, topic, and level of detail. In general, a chart needs to be large enough to comfortably display the details you need to include and expansive enough for the interactivity you want to support. For example, you always want to make it easy for people to read a chart's details and descriptive text — like labels and annotations — but you might also want to give people enough room to change the scope of a chart or investigate the data from different perspectives. On the other hand, you might want to use a small chart to offer glanceable information about an individual item or to provide a snapshot or preview of a larger version of the chart that people can reveal in a different view.

Prefer consistency across multiple charts, deviating only when you need to highlight differences. If multiple charts in your app serve a similar purpose, you generally don't want to imply that the charts are unrelated by using a different type or style for each one. Also, using a consistent visual approach for the charts in your app lets people use what they learn about one chart to help them understand another. Consider using different chart types and styles when you need to highlight meaningful differences between charts.

Maintain continuity among multiple charts that use the same data. When you use multiple charts to help people explore one dataset from different perspectives, it's important to use one chart type and consistent colors, annotations, layouts, and descriptive text to signal that the dataset remains the same. For example, the Health Trends screen shows small charts that each use a specific visual style to depict a recent trend in an area like steps or resting heart rate. When people choose a chart to reveal all their data in that area, the expanded version uses the same style, colors, marks, and annotations to strengthen the relationship between the versions.

#### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, visionOS, or watchOS.

#### Change log

| Date | Changes |
|------|---------|
| September 23, 2022 | New page. |

---

### Collaboration and sharing
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/collaboration-and-sharing

#### Hero image
![Collaboration and sharing](images/patterns-collaboration-and-sharing-intro@2x.png)
*A sketch of a person with an overlapping checkmark, suggesting effective collaboration. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

System interfaces and the Messages app can help you provide consistent and convenient ways for people to collaborate and share. For example, people can share content or begin a collaboration by dropping a document into a Messages conversation or selecting a destination in the familiar share sheet.

After a collaboration begins, people can use the Collaboration button in your app to communicate with others, perform custom actions, and manage details. In addition, people can receive Messages notifications when collaborators mention them, make changes, join, or leave.

You can take advantage of Messages integration and the system-provided sharing interfaces whether you implement collaboration and sharing through CloudKit, iCloud Drive, or a custom solution. To offer these features when you use a custom collaboration infrastructure, make sure your app also supports universal links (for developer guidance, see Supporting universal links in your app).

In addition to helping people share and collaborate on documents, visionOS supports immersive sharing experiences through SharePlay. For guidance, see SharePlay.

#### Best practices

Place the Share button in a convenient location, like a toolbar, to make it easy for people to start sharing or collaborating. In iOS 16, the system-provided share sheet includes ways to choose a file-sharing method and set permissions for a new collaboration; iPadOS 16 and macOS 13 introduce similar appearance and functionality in the sharing popover. In your SwiftUI app, you can also enable sharing by presenting a share link that opens the system-provided share sheet when people choose it; for developer guidance, see ShareLink.

If necessary, customize the share sheet or sharing popover to offer the types of file sharing your app supports. If you use CloudKit, you can add support for sending a copy of a file by passing both the file and your collaboration object to the share sheet. Because the share sheet has built-in support for multiple items, it automatically detects the file and makes the "send copy" functionality available. With iCloud Drive, your collaboration object supports "send copy" functionality by default. For custom collaboration, you can support "send copy" functionality in the share sheet by including a file — or a plain text representation of it — in your collaboration object.

Write succinct phrases that summarize the sharing permissions you support. For example, you might write phrases like "Only invited people can edit" or "Everyone can make changes." The system uses your permission summary in a button that reveals a set of sharing options that people use to define the collaboration.

Provide a set of simple sharing options that streamline collaboration setup. You can customize the view that appears when people choose the permission summary button to provide choices that reflect your collaboration functionality. For example, you might offer options that let people specify who can access the content and whether they can edit it or just read it, and whether collaborators can add new participants. Keep the number of custom choices to a minimum and group them in ways that help people understand them at a glance.

Prominently display the Collaboration button as soon as collaboration starts. The system-provided Collaboration button reminds people that the content is shared and identifies who's sharing it. Because the Collaboration button typically appears after people interact with the share sheet or sharing popover, it works well to place it next to the Share button.

Provide custom actions in the collaboration popover only if needed. Choosing the Collaboration button in your app reveals a popover that consists of three sections. The top section lists collaborators and provides communication buttons that can open Messages or FaceTime, the middle section contains your custom items, and the bottom section displays a button people use to manage the shared file. You don't want to overwhelm people with too much information, so it's crucial to offer only the most essential items that people need while they use your app to collaborate. For example, Notes summarizes the most recent updates and provides buttons that let people get more information about the updates or view more activities.

If it makes sense in your app, customize the title of the modal view's collaboration-management button. People choose this button — titled "Manage Shared File" by default — to reveal the collaboration-management view where they can change settings and add or remove collaborators. If you use CloudKit sharing, the system provides a management view for you; otherwise, you create your own.

Consider posting collaboration event notifications in Messages. Choose the type of event that occurred — such as a change in the content or the collaboration membership, or the mention of a participant — and include a universal link people can use to open the relevant view in your app. For developer guidance, see SWHighlightEvent.

#### Platform considerations

No additional considerations for iOS, iPadOS, or macOS. Not available in tvOS.

**visionOS**

By default, the system supports screen sharing for an app running in the Shared Space by streaming the current window to other collaborators. If one person transitions the app to a Full Space while sharing is in progress, the system pauses the stream for other people until the app returns to the Shared Space. For guidance, see Immersive experiences.

**watchOS**

In your SwiftUI app running in watchOS, use ShareLink to present the system-provided share sheet.

#### Change log

| Date | Changes |
|------|---------|
| December 5, 2023 | Added artwork illustrating button placement and various types of collaboration permissions. |
| June 21, 2023 | Updated to include guidance for visionOS. |
| September 14, 2022 | New page. |

---

### Drag and drop
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/drag-and-drop

#### Hero image
![Drag and drop](images/patterns-drag-and-drop-intro@2x.png)
*A sketch of a hand dragging a document, suggesting drag-and-drop interaction. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Drag and drop lets people move or copy content by dragging it from one location and dropping it in another. The feature uses standard gestures — a long press followed by a drag on touchscreen devices, a mouse-button drag on pointer-based devices — which makes it familiar and learnable. Because drag and drop operates on content — rather than requiring people to select content and use a menu, toolbar, or keyboard shortcut — it can help people perform tasks more efficiently.

Drag and drop can work within a single app or across apps on the same device. For example, in iPad someone might drag and drop an image from Photos into an email they're composing in Mail. On Mac, people can drag and drop files between apps in their windows, or use the desktop as a staging area for content.

#### Best practices

Support drag and drop for content that people frequently copy or move. Drag and drop is most useful when it's part of the natural flow of people's work, so focus on supporting it for items that people use in their day-to-day work.

Accept a variety of data types. The more data types your app can accept as a drop target, the more ways people can use drag and drop with your app. Consider accepting all data types that your app can reasonably display or process.

Support multiple simultaneous drags when it makes sense. Although supporting multiple simultaneous drags requires more development effort, the results help people work more efficiently because they can move several items at the same time. For example, people can select multiple photos in Photos and drag them all into an email.

Consider providing an alternative to drag and drop for copying and moving content. Although drag and drop can be efficient for people who discover it, it's not always obvious that it exists. Support it as a shortcut, but don't require it to perform tasks that are important to your app's experience.

Preserve transparency of operations by restoring content if the drag fails. If someone attempts a move by dragging content away from its original location, but the drop fails, restore the content to its original location so that people don't lose any data.

#### Providing feedback

Animate the content as the drag gesture begins. A subtle visual effect — like making the image slightly translucent — conveys that the item is in motion and helps people confirm that they initiated the drag. For example, you might let the original image fade while displaying a translucent version of the image that follows the pointer.

Update the drag image when people pause over a drop target. When someone drags content over a drop target, you can change the drag image to give people a preview of what the drop will look like. For example, if someone drags a photo over a Mail message, you might update the drag image to show how the photo would appear in the message.

Make drop targets apparent. People need to know where they can drop the item they're dragging. You can make drop targets apparent by making them stand out visually when the drag enters the target, or by making them visible at all times. If an area doesn't accept drops, consider displaying a badge on the drag image that indicates this.

#### Accepting drops

Allow people to drop items in their preferred location. In general, support dropping into specific locations within a view — like a particular position in a list or between images in a photo grid — rather than requiring people to drop items in a general region. Consider whether people need to insert content or replace existing content.

Provide appropriate drop feedback for your context. For example, when someone drags content into a document or list, consider showing the item appear in the list or a cursor in the document. When someone drags content to replace an item, consider highlighting the item so people understand it will be replaced.

#### Platform considerations

**iOS and iPadOS**

In iOS, drag and drop works within a single app. In iPadOS, drag and drop can occur within a single app or between apps. In both cases, SwiftUI and UIKit support drag-and-drop interactions.

**macOS**

On macOS, drag and drop works across apps and also between an app and the Finder. A successful drag is always assumed to be a move unless the user holds down the Option key to request a copy.

**visionOS**

In visionOS, drag and drop works within a single app and also within the same window. People can drag and drop by using indirect or direct touch, and by using the pointer (when one is available).

#### Change log

| Date | Changes |
|------|---------|
| June 21, 2023 | Updated to include guidance for visionOS. |
| September 14, 2022 | New page. |

---

### Entering data
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/entering-data

#### Hero image
![Entering data](images/patterns-entering-data-intro@2x.png)
*A sketch of a text cursor in a text field, suggesting data entry. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Entering data is a fundamental task in many apps, and doing it well can make the difference between a frustrating and a satisfying experience. While you can't always avoid requiring people to enter data, you can minimize the burden and make the process as smooth as possible.

#### Best practices

Minimize the amount of data people need to enter. Wherever possible, prefill fields with default values, infer data from context, and use pickers instead of free-form text fields. For example, if you can determine someone's city and state from their ZIP code, you can automatically populate those fields.

Use the right keyboard type for the data you're requesting. If you ask for a phone number, display a phone keypad; for an email, show a keyboard that includes at-sign and dot. Configuring the right keyboard prevents unnecessary mode changes and helps people enter data more quickly.

Validate data at the appropriate time. Catching errors early — such as when someone finishes a text field rather than waiting until they try to submit a form — lets people make corrections right away. However, avoid validating data as people are still typing: it can feel intrusive and disruptive.

Clearly indicate required fields. Use a visual indicator — such as an asterisk or the word "required" — to show which fields must be completed before people can proceed. Avoid designing forms where all fields are required; instead, require only what's necessary.

Use push notifications and other cues to remind people to complete partially filled forms, if appropriate. People can be distracted or need to pause, and a helpful reminder can make it easy to pick up where they left off.

Help people enter complex or unfamiliar data. For example, if someone needs to enter a credit card number, displaying the keyboard as soon as they activate the field, grouping the digits, and providing a camera option for scanning can all reduce friction. Show a sample of the expected format if it helps.

Offer autofill and autocomplete when appropriate. Suggestions save time and reduce errors. On iOS and iPadOS, the system can suggest contact details, passwords, and other commonly entered data.

#### Platform considerations

**macOS**

On macOS, people expect to be able to use keyboard shortcuts and standard text-editing commands when entering data. Make sure your text fields support standard editing shortcuts.

#### Change log

| Date | Changes |
|------|---------|
| September 14, 2022 | New page. |

---

### Feedback
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/feedback

#### Hero image
![Feedback](images/patterns-feedback-intro@2x.png)
*A sketch of a checkmark in a speech bubble, suggesting confirmation and feedback. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Feedback helps people know what's happening in your app, understand the results of their actions, and know what they can do next. Well-designed feedback is informative without being intrusive.

#### Best practices

Communicate status and results clearly. Use appropriate visual, auditory, or haptic feedback to confirm that an action was completed, highlight errors or warnings, and show progress. For example, show a checkmark after saving a file, display an error alert when something goes wrong, or use a progress indicator while loading content.

Provide feedback at the right moment. Confirmation feedback is most meaningful immediately after an action completes. Delayed feedback can be confusing because people may have moved on and lost context. At the same time, avoid overwhelming people with feedback for routine actions — not every tap needs a response beyond the visual change it causes.

Make error messages constructive. When something goes wrong, explain what happened in plain language and tell people how to resolve the problem. Avoid technical jargon and vague messages like "An error occurred." Instead, say something like "Your photo couldn't be uploaded. Check your internet connection and try again."

Use system-provided feedback mechanisms when possible. System-provided alerts, notifications, activity indicators, and haptic patterns are familiar to users and consistent with the platform's look and feel.

Match feedback intensity to the importance of the event. Reserve prominent feedback — like modal alerts — for important situations that require people to respond. Use subtle indicators — like brief animations or status bar changes — for routine updates. Overusing prominent feedback trains people to ignore it.

Avoid unnecessary interruptions. Feedback that requires people to stop what they're doing should be reserved for cases where it's truly necessary. Prefer non-interruptive patterns like banners, toasts, or subtle animations for informational messages.

#### Platform considerations

**watchOS**

On Apple Watch, consider using haptic feedback as the primary way to notify people of events, especially those that don't require them to look at the screen. The Apple Watch Taptic Engine provides haptic feedback that can communicate information without sound.

---

### File management
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/file-management

#### Hero image
![File management](images/patterns-file-management-intro@2x.png)
*A sketch of a document with a folded corner, suggesting a file. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Many apps need to help people create, open, save, and organize files. Apple platforms provide system frameworks and interfaces that handle most file management tasks, so your app can focus on working with the content itself. Letting the system handle file management also gives people a consistent experience across apps.

#### Creating and opening files

Use standard document picker interfaces when you need to help people browse and open files. The system document picker lets people navigate iCloud Drive and other locations, making it easy to find and open files across your app and others.

Support the file formats that are most relevant to your app's purpose. If your app works with a specific type of content — like images, audio files, or text documents — accept the file formats that people commonly use for that content type.

Display a meaningful default name for new documents. When people create a new file, show a name that reflects the content type, and make it easy for them to rename it. The name appears in the title bar, navigation bar, or other prominent locations, so it should be clear and helpful.

#### Saving work

Save content automatically when possible. People expect apps to preserve their work without requiring manual saves. Auto-saving lets people focus on their content rather than worrying about losing changes.

Don't require people to name a file before they can start working with it. Let people get started right away, and save with a default name. Provide easy ways to rename later.

Confirm before overwriting an existing file with a different name. If someone tries to save a document using the name of an existing file, make sure they understand what will happen before you replace the existing file.

#### Quick Look previews

Support Quick Look to help people preview files without opening them. When your app creates files that other apps might want to preview, implementing Quick Look support means people can see a preview in the Files app, in email, and in other contexts without needing to open your app.

#### Platform considerations

**iOS and iPadOS**

**Document launcher**

If your app is document-based, consider providing a document launcher — a home screen for your app that helps people find and manage their documents. A good document launcher makes it easy to create new documents, browse recent files, and organize existing ones.

**File provider**

Implement a File Provider extension if you store files in a custom location, such as on your own servers. A File Provider extension lets users browse and access those files through the system document picker, just like files in iCloud Drive.

**macOS**

**Custom file management**

macOS apps that manage files have more flexibility to implement custom file management interfaces. For example, you might provide a custom browser that shows files in a way that's tailored to your app's content type.

**Finder Sync**

Implement a Finder Sync extension if you have a file-synchronization service. Finder Sync lets you add badge icons, toolbar buttons, and contextual menu items to Finder windows, providing an integrated experience for managing synchronized files.

#### Change log

| Date | Changes |
|------|---------|
| September 14, 2022 | New page. |

---

### Going full screen
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/going-full-screen

#### Hero image
![Going full screen](images/patterns-going-full-screen-intro@2x.png)
*A sketch of a rectangle expanding to fill the frame, suggesting full-screen mode. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

A full-screen experience lets people immerse themselves in content by removing the surrounding interface elements — like navigation bars, toolbars, and status bars — that might otherwise compete for their attention. Full-screen mode is most valuable for content that benefits from the full display, such as photos, videos, maps, games, and documents. Many Apple apps, like Photos, Safari, and QuickTime Player, use full-screen mode.

#### Best practices

Make entering and exiting full screen easy and intuitive. People should be able to enter and exit full-screen mode with minimal effort. On devices that support it, respond to standard gestures or controls, and make sure people can always find a way to exit.

Display the most relevant content and controls when someone enters full screen. Because full-screen mode removes most UI chrome, you need to decide what — if anything — to show on top of the content. For media playback, for example, showing playback controls on tap is a common pattern. For documents and reading, hiding all controls by default (revealing them on tap or hover) is appropriate.

Preserve interface state when people enter and exit full screen. If someone is in the middle of reading a document or watching a video and goes full screen, make sure the content position is preserved so they can continue where they left off.

In general, hide the system UI when in full screen. In full screen, hide the status bar, navigation bars, toolbars, and any other interface elements that distract from the content. If some controls need to remain visible, use auto-hiding overlays that fade after a moment of inactivity and reappear when someone interacts with the screen.

Provide clear affordances for entering full screen. Don't rely on people to discover full-screen mode by accident. Consider adding a button that makes full-screen mode discoverable, and make it obvious when hovering or interacting with content that full-screen mode is available.

#### Platform considerations

**iOS and iPadOS**

On iPhone, most apps already occupy the entire screen, so "full screen" typically means hiding the status bar, navigation bars, and tab bars to maximize content space. This is most relevant for media playback and immersive experiences.

**macOS**

macOS has a built-in full-screen mode that removes the menu bar and Dock and lets a window occupy the entire display. Apps can enter native full-screen mode by implementing the NSWindow full-screen API.

#### Change log

| Date | Changes |
|------|---------|
| September 14, 2022 | New page. |

---

### Launching
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/launching

#### Hero image
![Launching](images/patterns-launching-intro@2x.png)
*A sketch of a rocket launching, suggesting app startup. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

App launch is the first experience people have with your app, and it sets the tone for everything that follows. A fast, smooth launch makes a great first impression and shows respect for people's time. A launch that's slow, disorienting, or requires several taps before people can do anything meaningful can undermine confidence in your app.

#### Best practices

Launch as fast as possible. People notice launch times, so do everything you can to minimize the time between when someone taps your app icon and when they can start using your app. Defer work that isn't required at startup, and avoid making network requests that delay the initial display.

Restore the previous state so people can continue where they left off. If someone was in the middle of a task when they last used your app, return them to that point. People shouldn't have to navigate back to where they were after a relaunch.

Avoid displaying a splash screen. A splash screen that displays only your app's logo delays the point at which people can start using your app. Instead, display a launch screen that matches your app's initial state, making the transition feel instantaneous.

Don't ask for permissions or setup steps at launch unless they're immediately necessary. Requesting permissions before people understand why they're needed often results in denial. Ask for permissions in context — when people are about to use a feature that requires them. Similarly, defer setup and configuration to the point where people actually need it.

Handle initialization gracefully. If there are tasks your app must complete before it can be used — like loading a local database or setting up a connection — show a meaningful loading indicator rather than a blank screen.

#### Launch screens

Provide a launch screen that matches your app's initial UI. The launch screen is displayed immediately when your app starts and replaced by your app's actual UI as soon as it's ready. Because the transition happens so quickly, the launch screen should look like a static version of your initial UI. Avoid including dynamic content, animations, or anything that might appear jarring.

Update your launch screen when you update your app's initial UI. If you redesign your app's home screen, update the launch screen to match. An outdated launch screen can briefly show outdated UI and create a disorienting flash.

#### Platform considerations

**iOS and iPadOS**

On iPhone, apps typically resume from the multitasking state rather than re-launching from scratch. However, your app might be terminated and relaunched if it hasn't been used recently. Make sure your launch experience is good for both scenarios.

**tvOS**

On Apple TV, app launch is often the first step of a living room experience. Launch quickly and start displaying meaningful content as soon as possible, since people are often at a distance from the screen.

**visionOS**

In visionOS, apps launch into the Shared Space and appear alongside other apps and the person's surroundings. Launch with a comfortable window size and placement that fits well in a shared spatial context.

#### Change log

| Date | Changes |
|------|---------|
| June 21, 2023 | Updated to include guidance for visionOS. |
| September 14, 2022 | New page. |

---

### Live-viewing apps
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/live-viewing-apps

#### Hero image
![Live-viewing apps](images/patterns-live-viewing-intro@2x.png)
*A sketch of a play button on a screen, suggesting live video viewing. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Live-viewing apps let people watch streaming video content like live sports, news, and events. These apps need to handle the unique challenges of live content — such as variable connection quality, lack of scrubbing, and real-time scheduling — while delivering an engaging viewing experience.

#### Best practices

Display live content without requiring people to navigate past unnecessary steps. People want to start watching immediately. Design a direct path from launch to playback, minimizing the number of screens or interactions required.

Make it clear that content is live. Use visual indicators — like a "LIVE" badge, a red dot, or a real-time program clock — to help people quickly see that what they're watching is happening right now. This is especially important for apps that also offer on-demand content.

Gracefully handle connection issues. Live streams can fail or degrade in quality due to network conditions. Provide clear feedback when buffering occurs, and offer options to switch to lower quality or try again. Avoid abrupt failures that leave people staring at a blank screen.

Support background audio when appropriate. If your live content includes audio that people might want to continue listening to while using other apps — like live radio or sports audio — support background playback.

Provide relevant live metadata. Display information that's specific to the live nature of the content, like the current program title, the time remaining, or the score for a live sports event.

#### EPG experience

An Electronic Program Guide (EPG) helps people browse current and upcoming content. If your app includes an EPG, design it to be easy to scan and navigate, especially from a distance (on Apple TV) or with one hand (on iPhone). Group content logically by channel, time, or category.

Make it easy to tune to a channel directly from the EPG. People should be able to tap a program in the EPG and immediately start watching without extra steps.

#### Cloud DVR

If your app supports cloud DVR, make it easy to find and manage recordings. Clearly differentiate recorded content from live content in your UI.

#### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, visionOS, or watchOS.

---

### Loading
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/loading

#### Hero image
![Loading](images/patterns-loading-intro@2x.png)
*A sketch of a circular progress indicator, suggesting loading state. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

When people have to wait for content to load or a task to complete, give them clear feedback about what's happening and how long it will take. A good loading experience keeps people informed without getting in their way.

#### Best practices

Minimize loading time whenever possible. Optimize your app to load content as fast as possible. Use techniques like caching, lazy loading, and prefetching to reduce wait times. When loading is unavoidable, show content progressively as it becomes available rather than waiting for everything to load before displaying anything.

Show activity only when something is actually happening. Don't display loading indicators when there's nothing to indicate. If your app is idle and waiting for user input, don't show a spinner.

Use determinate progress indicators when you know how long a task will take. A progress bar or percentage that advances toward completion helps people understand how long they'll have to wait and builds confidence that something is actually happening. Use indeterminate indicators only when you genuinely don't know how long something will take.

Display a loading indicator while content loads in place. When content is being loaded into an area that's already visible — like a scroll view or a table — show an indicator within that area rather than blocking the entire UI. This lets people continue interacting with the rest of your app.

Consider showing placeholder content while loading. Skeleton screens or placeholder views that approximate the shape of the incoming content reduce the jarring transition from blank to full content and help people understand the layout before the real data arrives.

Provide meaningful error messages if loading fails. If loading fails, don't just show a generic error. Explain what happened and offer actionable recovery steps, like retrying the operation or checking the network connection.

#### Showing progress

When showing progress for a long operation, provide enough information to help people gauge how long the operation will take. If the operation has identifiable stages, consider showing the current stage name along with the progress indicator.

For multi-step downloads or installations, show overall progress rather than individual step progress when possible. People care more about when the whole operation will complete than the breakdown of individual steps.

#### Platform considerations

**watchOS**

On Apple Watch, loading operations should complete as quickly as possible. If a loading operation takes more than a few seconds, consider redesigning the operation or deferring it to the background. People use Apple Watch for quick interactions and extended waiting is particularly unwelcome.

#### Change log

| Date | Changes |
|------|---------|
| September 14, 2022 | New page. |

---

### Managing accounts
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/managing-accounts

#### Hero image
![Managing accounts](images/patterns-managing-accounts-intro@2x.png)
*A sketch of a person silhouette in a circle, suggesting a user account. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Many apps require people to create accounts so they can access personalized content or services. While accounts can provide great value, they add friction to your app experience. Follow best practices to make account management as smooth as possible while respecting people's privacy.

#### Best practices

Avoid requiring account creation unless it's necessary. If people can use your app without an account — even in a limited way — let them. This is especially important for apps where the core experience doesn't need personalization or cloud storage. Requiring an account upfront is one of the most common reasons people abandon apps during onboarding.

Support Sign in with Apple when you require accounts. Sign in with Apple gives people a fast, easy way to create accounts using their existing Apple ID. It respects their privacy, supports two-factor authentication automatically, and lets people use their Apple ID that they already trust. If you offer other social login options, you must offer Sign in with Apple as well (per App Store guidelines).

Prefill account creation forms with information that you already have or can infer. For example, if someone is using a device and you can access their name from contacts or their email from settings (with permission), offer to prefill those fields.

Request only the information you need. Don't ask for information you don't use. Every field you add to an account creation form is another barrier to entry.

Clearly explain why you need personal information. If you request sensitive information like birthdate or phone number, explain why you need it and how it will be used.

Make account recovery easy. Provide clear, accessible ways for people to recover a lost password or regain access to their account.

#### Deleting accounts

Provide a straightforward way for people to delete their accounts. App Store guidelines require that apps that allow people to create accounts must also let them delete those accounts from within the app. The delete option should be easy to find, and the deletion process should be clear and honest about what data will be deleted.

Before finalizing account deletion, clearly explain what will be deleted. Give people the opportunity to export their data first, and consider a grace period during which they can reactivate the account.

#### TV provider accounts

If your app offers TV content, integrate with the system's TV provider authentication when possible. This lets people sign in with their TV provider credentials in a single place and have that authentication shared across all compatible apps, rather than signing in separately in every app.

#### Platform considerations

**tvOS**

On Apple TV, account sign-in is especially important because physical keyboard input is cumbersome. Prioritize Sign in with Apple and QR code flows, or let people authenticate on their iPhone or iPad using the TV Remote app or Continuity features.

**watchOS**

On Apple Watch, account management should be minimal. People should be able to sign in once on iPhone and have that carry over to the Watch app. Avoid requiring complex account management on the Watch itself.

---

### Managing notifications
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/managing-notifications

#### Hero image
![Managing notifications](images/patterns-managing-notifications-intro@2x.png)
*A sketch of a bell, suggesting notifications. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Notifications help people stay informed about updates and events that are important to them. However, receiving too many notifications — or notifications that aren't relevant — can frustrate people and lead them to disable notifications entirely. Design a thoughtful notification strategy that delivers genuine value.

#### Integrating with Focus

Focus is a system feature that lets people filter notifications based on what they're doing. Your app's notifications can integrate with Focus to respect people's preferences.

Apple defines several levels of interruption for notifications:

| Interruption level | Description |
|-------------------|-------------|
| Passive | Adds to the Notification Center without alerting |
| Active | Default level; sounds an alert |
| Time Sensitive | Breaks through certain Focus filters |
| Critical | Always delivered; requires special entitlement |

Assign the appropriate interruption level based on how urgent and important a notification is. Most notifications should use the Active or Passive levels. Time Sensitive notifications should only be used for notifications that are genuinely time-sensitive — like a ride arriving or a package being delivered. Critical notifications require an entitlement from Apple and should only be used for truly critical situations like severe weather alerts or medical reminders.

#### Best practices

Ask for notification permission in context, not at app launch. Request permission when someone is about to do something that benefits from notifications. Asking immediately at launch, before people understand the value, often results in denial. For example, if someone is about to start a workout, ask whether they want reminders about their workout goals.

Clearly explain the value of each type of notification. Before requesting permission, tell people what kinds of notifications they'll receive and why those notifications will be useful. Be specific: "We'll notify you when a friend starts a game" is more compelling than "Allow notifications?"

Let people opt in to specific notification types. Rather than requesting blanket permission for all notifications, consider letting people customize which types of notifications they receive. This reduces the likelihood of people disabling all notifications after they get one they don't want.

Provide easy access to notification settings. Make it easy for people to adjust their notification preferences from within your app, so they can fine-tune what they receive without navigating to system Settings.

Avoid sending redundant or excessive notifications. If someone has already seen information in your app, don't send a notification about it. And don't send multiple notifications that convey the same information.

#### Sending marketing notifications

Only send marketing notifications to people who have explicitly opted in to receive them. Don't use notification permissions granted for transactional or functional notifications to send marketing messages.

Make it easy to opt out of marketing notifications. People should be able to unsubscribe from marketing notifications without unsubscribing from all notifications.

#### Platform considerations

**watchOS**

On Apple Watch, notifications have additional prominence because they arrive as taps on the wrist. Design notification content that is clear and useful at a glance. Use short notification titles and bodies so people can understand the notification without detailed reading.

---

### Modality
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/modality

#### Hero image
![Modality](images/patterns-modality-intro@2x.png)
*A sketch of a sheet rising from the bottom of a screen, suggesting a modal presentation. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Modality is a design technique that presents content in a temporary mode that requires an explicit action to exit. When presented modally, content is:

- Focused: Modal content temporarily prevents interaction with the previous context, helping people focus on completing a self-contained task.
- Temporary: People must explicitly complete or dismiss the modal context before returning to the previous one.
- Distinct: Modal content appears above the previous view, clearly indicating that it's a temporary departure from the normal navigation flow.

Use modality to present self-contained tasks that require an action to complete (such as creating a new item), to request a required choice or required acknowledgement (such as an alert), and to display content that people need to view without leaving the current context.

#### Best practices

Use modal presentations sparingly. Because modality interrupts the flow of your app and requires people to explicitly close or complete the modal view, overuse can feel disruptive and exhausting. Reserve modal presentations for situations where interruption is genuinely warranted.

Always provide a way to dismiss a modal view. People should always be able to close a modal view without completing the task, unless completing the task is truly required. Provide a close or cancel button, and support standard gestures like swipe-to-dismiss.

Make the purpose of a modal view clear. The title of a modal sheet or the headline of an alert should immediately convey what the modal is for and what action is being requested.

Avoid nesting modals. Presenting a modal view from within another modal view creates a confusing experience. If you find yourself nesting modals, reconsider your navigation structure.

Avoid using alerts for confirmation of routine operations. Alerts are meant for urgent or important information that requires an immediate decision. Using them for routine actions desensitizes people and makes truly important alerts feel less urgent.

#### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, visionOS, or watchOS.

#### Change log

| Date | Changes |
|------|---------|
| December 5, 2023 | Added guidance on using modality appropriately in visionOS. |
| June 21, 2023 | Updated to include guidance for visionOS. |

---

### Multitasking
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/multitasking

#### Hero image
![Multitasking](images/patterns-multitasking-intro@2x.png)
*A sketch of two overlapping windows, suggesting multiple apps running simultaneously. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Multitasking lets people use multiple apps at once, switch quickly between apps, and get more done. Apple platforms support different forms of multitasking, each tailored to the capabilities and form factor of the device.

#### Best practices

Support multitasking to give people more flexibility. People often want to use multiple apps simultaneously or switch between them frequently. An app that works well in multitasking scenarios enhances its usefulness.

Adapt your layout for different multitasking sizes. On iPadOS, your app might appear in a range of sizes, from full screen to a narrow split-view column. Design your interface to work well at a variety of widths rather than assuming a specific size.

Preserve state when people switch away from your app. When someone switches to another app and comes back, they should find your app in the state they left it. Don't reset the interface or lose their progress.

Continue important tasks in the background. Audio playback, navigation, downloads, and other ongoing activities should continue even when your app isn't in the foreground. Use background modes appropriately to ensure a continuous experience.

#### Platform considerations

**iOS**

iOS supports multitasking through the App Switcher, which lets people switch between apps, and through Picture-in-Picture for video playback. Apps run in the foreground or are suspended in the background.

**iPadOS**

iPadOS offers the richest multitasking support, including Split View (two apps side by side), Slide Over (a floating panel), Stage Manager (a window management interface), and Center Window. Design your app to work well in all of these modes.

**macOS**

macOS supports traditional window-based multitasking, where multiple apps run in separate windows and people switch between them with the Dock or keyboard shortcuts. macOS also supports Spaces, Mission Control, and full-screen mode.

**tvOS**

tvOS doesn't support traditional multitasking between apps. However, apps can support Picture-in-Picture for video playback.

**visionOS**

In visionOS, apps can run side by side in the Shared Space, and people can open multiple windows from a single app. Apps can also request a Full Space for immersive experiences.

#### Change log

| Date | Changes |
|------|---------|
| June 9, 2025 | Updated to include guidance for Stage Manager and Center Window on iPadOS. |
| December 5, 2023 | Updated to include guidance for visionOS. |
| June 21, 2023 | Updated to include guidance for visionOS. |

---

### Offering help
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/offering-help

#### Hero image
![Offering help](images/patterns-offering-help-intro@2x.png)
*A sketch of a question mark in a speech bubble, suggesting help or support. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Well-designed apps are intuitive enough that most people can figure them out without explicit instructions. However, some apps — especially those with complex features or workflows — benefit from offering contextual help to guide people when they need it.

#### Best practices

Make your app self-explanatory whenever possible. The best help is an interface that doesn't need explanation. Use clear labels, familiar metaphors, and consistent patterns so people can understand your app through exploration.

Offer help contextually, not as a last resort. Contextual help appears near the UI element it describes, in the moment when someone needs it. Inline tips, tooltips, and coach marks are more effective than a separate help document because they're delivered at the right time and place.

Make help discoverable without making it intrusive. People who need help should be able to find it easily, but people who don't need help shouldn't be bothered by it. Consider offering a subtle help button or an optional tutorial that people can invoke when they want it.

Keep help concise. Long explanations are rarely read. Use plain language, focus on the most important information, and link to more detailed resources for advanced topics.

#### Creating tips

Use TipKit to create system-styled tips. TipKit is a framework that helps you create contextual tips that match the style of system-provided tips. Tips created with TipKit can sync across devices using iCloud and respect the system's display frequency rules.

Design tips to answer specific questions. A good tip explains something actionable: what a control does, how to perform a specific action, or when a feature is most useful. Avoid writing vague tips like "Tap here to learn more."

#### Platform considerations

No additional considerations for iOS, iPadOS, tvOS, or watchOS.

**macOS and visionOS**

On macOS, tooltips are the primary way to provide help for interface elements. A tooltip appears when someone hovers the pointer over a control for a moment and displays a brief label or description. Write tooltip text in a way that completes the phrase "This button…" by describing the button's action. Keep tooltip text short — one short sentence or phrase is ideal.

In visionOS, tooltips appear when someone looks at a control and remain visible briefly. Follow similar guidance as macOS.

#### Change log

| Date | Changes |
|------|---------|
| December 5, 2023 | Added guidance on using TipKit to create contextual tips. |
| September 12, 2023 | Added guidance for macOS and visionOS tooltips. |

---

### Onboarding
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/onboarding

#### Hero image
![Onboarding](images/patterns-onboarding-intro@2x.png)
*A sketch of a person stepping through a doorway, suggesting entry or onboarding. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Onboarding introduces people to your app and helps them start using it effectively. A great onboarding experience gets people up and running quickly without overwhelming them with information they don't need yet.

#### Best practices

Get people to the core experience as fast as possible. People want to see what your app can do, not read about it. Defer non-essential information, setup steps, and introductory screens in favor of letting people start using the app.

Avoid lengthy tutorials. Most people don't read long tutorials. Instead, teach the essential features through use — let people discover features as they need them, and offer contextual help when appropriate.

Use onboarding to address genuine first-use challenges. If there are things people must know before using a core feature — like granting location permission for a navigation app — onboarding is the right place to address them. But limit this to genuinely essential steps.

Make it easy to skip or dismiss onboarding. Some people don't need an introduction. Give them a quick way to skip to the main interface.

#### Additional content

If your app has content that needs to be set up or configured before people can use it — like choosing topics or importing data — make this process feel helpful rather than burdensome. Show progress, and let people start using the app even if setup isn't complete.

#### Additional requests

Don't front-load permission requests. Ask for permissions only when people reach a feature that needs them, so the request has context. If your app requests multiple permissions, stagger them — asking for several at once feels overwhelming.

Explain each permission before requesting it. A brief, plain-language explanation of why you need a permission increases the likelihood that people will grant it.

#### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, visionOS, or watchOS.

#### Change log

| Date | Changes |
|------|---------|
| June 10, 2024 | Updated guidance on permissions and setup during onboarding. |
| June 21, 2023 | Updated to include guidance for visionOS. |

---

### Playing audio
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/playing-audio

#### Hero image
![Playing audio](images/patterns-playing-audio-intro@2x.png)
*A sketch of a speaker with sound waves, suggesting audio playback. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

**Silence**

The Ring/Silent switch on iPhone (and the equivalent settings on other devices) lets people silence all audio. An app should respect this setting: if the Ring/Silent switch is in silent mode, your app's audio — except for alarms, audio specifically requested by the user (like a song in a music player), and accessibility audio — should be silent.

**Volume**

Respect the system volume setting. Don't set the system volume programmatically unless your app includes a mixer or other audio controls. Use the system volume controls instead.

**Headphones**

When headphones are disconnected, audio output moves to the speaker. If your app is playing audio that should stop when headphones are disconnected — like a private audio playback — pause playback when this happens.

#### Best practices

Choose the appropriate audio session category for your use case. The audio session category determines how your app's audio interacts with other audio on the device and with the Ring/Silent switch.

| Category | Description |
|----------|-------------|
| Ambient | Your audio plays alongside other audio; silenced by the Ring/Silent switch |
| Solo Ambient | Like Ambient, but silences other audio |
| Playback | Intended for audio that should continue in the background and not be silenced by the Ring/Silent switch |
| Record | For audio recording |
| Play and Record | For simultaneous recording and playback |
| Multi-Route | For routing audio to multiple outputs |

Match your audio category to the expectations people have for your app. A meditation app playing background music should use Ambient; a podcast app should use Playback so audio continues in the background.

Support Now Playing and the system media controls. If your app plays audio, implement Now Playing metadata and respond to remote controls (play, pause, skip) so people can control playback from Control Center, the Lock Screen, and external hardware like headphones.

Provide visual feedback that mirrors the audio experience. Show what's playing, display progress, and update the interface when playback state changes (playing, paused, stopped).

Duck audio appropriately. If your app plays background audio alongside other audio — like background music in a game — use the system's audio ducking feature to lower the volume of your audio when other apps need to be heard.

#### Handling interruptions

Handle audio interruptions gracefully. Audio is frequently interrupted by phone calls, alarms, and other apps. When an interruption begins, pause your audio. When the interruption ends, decide whether to resume based on your app type — a music player should resume, but a guided meditation app might not.

Save playback state when interrupted. If someone is in the middle of a podcast or audiobook and gets a phone call, they should be able to return to exactly where they were after the call ends.

#### Platform considerations

**iOS and iPadOS**

On iOS and iPadOS, the Ring/Silent switch determines whether ambient audio plays. Use the appropriate audio session category and configure audio session options for route changes.

**macOS**

On macOS, there is no Ring/Silent switch. Audio plays based on the system volume and mute settings. Implement the Now Playing controls so your app integrates with the media key on keyboard.

**tvOS**

On Apple TV, audio playback should continue seamlessly even when people navigate the system interface. Implement AVKit for media playback to get automatic integration with system playback controls.

**visionOS**

In visionOS, audio is spatialized and people can hear audio from multiple apps simultaneously. Design your audio with spatial audio in mind, and make sure your app's audio plays well alongside other spatial experiences.

**watchOS**

On Apple Watch, audio can play through the built-in speaker or through paired Bluetooth headphones. Background audio plays when the app is not in the foreground.

#### Change log

| Date | Changes |
|------|---------|
| June 21, 2023 | Updated to include guidance for visionOS. |

---

### Playing haptics
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/playing-haptics

#### Hero image
![Playing haptics](images/patterns-playing-haptics-intro@2x.png)
*A sketch of a hand with vibration waves, suggesting haptic feedback. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Haptics provide tactile feedback that adds a physical dimension to the user experience. On supported Apple devices, haptics can make interactions feel more immediate, confirm actions, and communicate the result of gestures or system events.

#### Best practices

Use haptics to enhance existing visual or auditory feedback, not to replace it. Haptics should complement your app's other feedback mechanisms. People in noisy environments or with hearing difficulties benefit from haptics, but haptics alone shouldn't be the only way you communicate important information.

Align haptics with visual and auditory feedback. A haptic should occur at the same time as the visual or auditory event it corresponds to. Misaligned feedback feels jarring and confusing.

Use system haptics when they fit your use case. System haptics are tuned for specific interaction types and are familiar to users. Custom haptics require more development effort and should be used only when system haptics don't fit your needs.

Avoid overusing haptics. Haptics are most effective when used selectively for meaningful moments. If every action produces a haptic, they lose their significance and become noise.

Don't play haptics for every interaction. Reserve haptics for actions that have a meaningful result, not just any tap or swipe.

#### Custom haptics

Design custom haptic patterns to match the feel of your content. If your app involves physical interactions — like a game with impact events, or a music app where people feel the beat — custom haptic patterns can enhance the experience. Use Core Haptics to design patterns that feel natural for your context.

Test haptics on physical devices. Haptics feel different on different devices and can't be simulated accurately in the simulator. Always test your haptic patterns on real hardware.

#### Platform considerations

**iOS**

iOS supports three categories of system haptics:

**Notification haptics**

Communicate the result of a task or action.

**Impact haptics**

Indicate that a collision or other impact occurred.

**Selection haptics**

Indicate that a selection is changing.

**macOS**

macOS supports haptics through the Force Touch trackpad. The system provides haptic patterns for standard interactions. Apps can request custom haptic patterns through NSHapticFeedbackManager.

| Haptic pattern | When to use |
|----------------|-------------|
| Generic | For any haptic feedback not covered by the other patterns |
| Alignment | When an element aligns with another element |
| Level change | When a level indicator or slider value changes |

**watchOS**

Apple Watch's Taptic Engine provides haptic feedback for notifications and app-defined events. Use the WKInterfaceDevice taptic method for playback patterns.

#### Change log

| Date | Changes |
|------|---------|
| May 7, 2024 | Added guidance on custom haptics and the Core Haptics framework. |
| June 21, 2023 | Updated to include guidance for visionOS. |

---

### Playing video
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/playing-video

#### Hero image
![Playing video](images/patterns-playing-video-intro@2x.png)
*A sketch of a play button on a video frame, suggesting video playback. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Video playback is a central experience in many apps, from streaming media to tutorials and product videos. Apple's AVKit framework and system-provided playback controls give people a consistent, polished experience across all Apple platforms.

#### Best practices

Use system-provided playback controls whenever possible. System controls provide familiar, consistent interactions and integrate automatically with the Lock Screen, Control Center, external hardware, and other system features. Custom playback controls require more work and can feel unfamiliar to users.

Support Picture-in-Picture where appropriate. People appreciate being able to continue watching a video while using other apps. Implement Picture-in-Picture for any long-form video content.

Provide alternative content for situations where video can't play. If someone is offline or in a low-bandwidth environment, offer a meaningful fallback rather than a blank or broken player.

Support standard media key interactions. People expect to be able to use the media keys on external keyboards, headphones, and other hardware to control video playback.

Make playback resumable. If someone pauses video or leaves your app, preserve their position so they can resume watching from where they left off.

#### Integrating with the TV app

If your app offers video content, integrate with the Apple TV app to let people browse and watch your content without leaving the TV app experience.

**Loading content**

Implement SKAdNetwork and TVTopShelf to surface content in the TV app. Make sure your content is tagged with appropriate metadata so the TV app can display it accurately.

**Exiting playback**

When playback ends, handle the exit cleanly. Return people to your app's content browser rather than leaving them on a black screen or navigating to a confusing destination.

#### Platform considerations

**tvOS**

On Apple TV, video playback is the primary experience. Use AVKit's TVMediaPlayerViewController or AVPlayerViewController for a system-consistent playback experience. Design for a 10-foot viewing distance with large controls and clear visual feedback.

**visionOS**

In visionOS, video can be played in a floating window or in an immersive environment. Use the system video player to support both contexts automatically. Consider implementing immersive video experiences using RealityKit for 3D and 180°/360° video content.

**watchOS**

Apple Watch supports short video clips. Provide encoded content in a format suited for the Watch's smaller display and storage constraints.

| Encoding setting | Recommended value |
|-----------------|-------------------|
| Format | H.264 |
| Maximum bit rate | 40 Mbps |
| Maximum frame rate | 30 fps |
| Maximum resolution | 1920 x 1080 |

#### Change log

| Date | Changes |
|------|---------|
| September 12, 2023 | Added guidance for visionOS video playback. |
| June 21, 2023 | Updated to include guidance for visionOS. |

---

### Printing
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/printing

#### Hero image
![Printing](images/patterns-printing-intro@2x.png)
*A sketch of a printer, suggesting print functionality. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Printing gives people a way to produce physical copies of content from your app. On Apple platforms, the system print panel provides a consistent printing interface that handles printer selection, paper size, orientation, and other options. Your app's job is to generate appropriate print content and present the print panel at the right time.

#### Best practices

Use the system print panel. Don't build your own print UI — use the system print panel instead. It's familiar to users, handles a wide variety of printers automatically, and provides a consistent experience across apps.

Format print output for paper. Screen layouts often don't translate well to paper. Design a print-specific layout that accounts for the different proportions, the absence of color on some printers, and the need to include information (like a URL or timestamp) that's implicit on screen but necessary on paper.

Make printing easy to find. Add a Print option to your app's Share menu, Action menu, or the File menu on macOS. People should be able to find the print option without hunting for it.

Support printing of content that people would actually want to print. Focus on the types of content in your app that make sense in physical form — documents, recipes, maps, boarding passes, and so on.

Preview the print output before printing. If your app generates content that might look different when printed, consider providing a preview. The system print panel includes a print preview on macOS.

#### Platform considerations

**macOS**

On macOS, printing is typically available through the File menu and is expected in any app that works with documents. The system print panel on macOS provides full access to printer options, page setup, and print preview.

---

### Ratings and reviews
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/ratings-and-reviews

#### Hero image
![Ratings and reviews](images/patterns-ratings-and-reviews-intro@2x.png)
*A sketch of five stars, suggesting a star rating. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Ratings and reviews help people make informed decisions about apps on the App Store. Well-timed, respectful rating prompts encourage people who are happy with your app to leave a rating, which benefits both your app's visibility and other people's decisions.

#### Best practices

Use the system rating prompt. The system provides a standard rating prompt through SKStoreReviewController that you can trigger programmatically. Using the system prompt ensures a consistent, trusted experience for people and ensures your app complies with App Store review guidelines.

Ask for a rating at the right time. Ask after people have had a chance to use your app enough to form an opinion — not on the first launch, not in the middle of a complex task, and not when people are likely to be frustrated. Good moments include after completing a task successfully, after using the app multiple times, or after a positive experience.

Don't ask for ratings too often. The system limits how often a rating prompt can appear. Even within those limits, being conservative about when you trigger the prompt improves the quality of ratings you receive.

Never ask people to leave a positive review or incentivize ratings. Asking people to leave a specific rating, or offering in-app rewards for doing so, violates App Store guidelines.

Don't prevent people from using your app while waiting for a rating. The system rating prompt is non-blocking, but any custom rating-related UI you build should not interrupt or block the main app experience.

#### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, visionOS, or watchOS.

#### Change log

| Date | Changes |
|------|---------|
| September 12, 2023 | Updated guidance on the timing of rating prompts. |

---

### Searching
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/searching

#### Hero image
![Searching](images/patterns-searching-intro@2x.png)
*A sketch of a magnifying glass, suggesting search functionality. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Search lets people quickly find specific content in your app. A well-implemented search experience is one of the most powerful tools for helping people navigate large datasets or libraries of content.

#### Best practices

Make search easy to access. Place the search field in a prominent, consistent location — at the top of a content view or in a navigation bar — so people know where to look for it. On iOS and iPadOS, the search field typically appears at the top of a list or content view, where people can reveal it by scrolling up or tapping a Search button.

Search as people type. Update results incrementally as people type, rather than requiring them to press a search button. Immediate results feel more responsive and help people refine their query based on what they see.

Display the most relevant results first. Order results by relevance, not just alphabetically or chronologically. Consider what people are most likely looking for and surface it prominently.

Show recent searches and suggestions. Before people start typing, display recent searches and popular suggestions to speed up the search process. If your app has predictive search capabilities, use them.

Handle empty states gracefully. When a search returns no results, explain why and offer helpful suggestions. "No results for 'xyz'. Check the spelling or try a different search term." is more helpful than just "No results."

Preserve the search state when people navigate to a result and come back. If someone taps a search result, explores it, and then comes back to the search view, their query and results should still be there.

#### Systemwide search

If your app has content that people would benefit from finding through Spotlight or Siri, implement Core Spotlight to index that content for systemwide search. This lets people find your content without first opening your app.

#### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, visionOS, or watchOS.

#### Change log

| Date | Changes |
|------|---------|
| June 9, 2025 | Added guidance on integrating with Spotlight and Siri for systemwide search. |

---

### Settings
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/settings

#### Hero image
![Settings](images/patterns-settings-intro@2x.png)
*A sketch of a gear, suggesting settings or configuration. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

People use settings to customize your app's behavior, configure features, and manage their account and preferences. How and where you expose settings has a significant impact on discoverability and how often people actually use them.

#### Best practices

Put the most important settings in your app, not only in the Settings app. Settings buried in the system Settings app are out of sight and out of mind. If a setting is important to the everyday use of your app, put it somewhere accessible in the app itself — a settings screen, a gear icon in the toolbar, or a contextual menu.

Use sensible defaults. Most people never change settings. Design your defaults to be the best choice for most people, so that the app works well out of the box.

Make settings changes take effect immediately. Don't require people to save or apply settings explicitly. Apply changes as people make them, giving immediate feedback about the result.

Avoid requiring people to restart your app to apply settings. Requiring a restart after changing a setting is disruptive. Design your app to apply settings dynamically.

Provide a search-optimized entry in the Settings app for complex configurations. If your app has many settings that benefit from the system's search and organization features, include a Settings bundle. This is particularly valuable for enterprise or professional apps.

#### General settings

Put general preferences — things like theme, language, and notification preferences — in a dedicated settings screen within your app.

#### Task-specific options

For options that apply to a specific task or context — like export settings for a document or display options for a map — present them inline with the task or in a contextual menu rather than a general settings screen.

#### System settings

Some settings, like location and camera access, are managed by the system in the Settings app. When your app needs permissions, follow the system patterns for requesting and explaining them. Provide a shortcut to the system Settings page for your app when relevant.

#### Platform considerations

**macOS**

On macOS, settings are typically accessed through the app's Settings menu item (previously named Preferences). Use SwiftUI's Settings scene or implement a settings window that conforms to macOS UI conventions.

**watchOS**

On Apple Watch, settings should be minimal. Use the companion iPhone app for complex settings. Put only the most essential, frequently-changed options directly on the watch.

#### Change log

| Date | Changes |
|------|---------|
| June 10, 2024 | Updated guidance on settings location and defaults. |

---

### Undo and redo
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/undo-and-redo

#### Hero image
![Undo and redo](images/patterns-undo-redo-intro@2x.png)
*A sketch of curved arrows suggesting undo and redo actions. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

Undo and redo let people reverse and reapply their most recent actions, giving them the confidence to explore and experiment without fear of making irreversible mistakes. Supporting undo is a core expectation in any app that lets people create or edit content.

#### Best practices

Implement undo and redo for all user-initiated actions that modify content. People expect to be able to undo almost any action in an editing context. Failing to support undo for a significant action — like deleting a large chunk of text, moving an important element, or changing a setting — can result in irreversible data loss.

Implement multi-level undo. Support undoing a sequence of actions, not just the most recent one. Most text editors and creative apps allow dozens or even hundreds of levels of undo.

Make undo and redo discoverable. On devices without a keyboard, undo may not be obvious. Consider adding Undo and Redo buttons to your toolbar for content-editing contexts.

Describe the action being undone in the menu item. On macOS and in the action menu, use the name of the action being undone — "Undo Delete" or "Undo Move" — rather than just "Undo." This helps people confirm what they're about to undo.

Clear the undo stack when appropriate. When someone saves a document (on macOS), opens a new document, or performs a major, explicitly irreversible action, clear the undo stack. Preserving undo history across major transitions can be confusing.

#### Platform considerations

Not supported in tvOS or watchOS.

**iOS and iPadOS**

On iOS, shake-to-undo is a traditional gesture for triggering undo. In addition, on iOS 13 and later, people can use a three-finger swipe left to undo and right to redo. In content-editing contexts, consider adding Undo and Redo buttons to the keyboard toolbar or elsewhere in your UI.

**macOS**

On macOS, Undo (Command-Z) and Redo (Shift-Command-Z) are expected keyboard shortcuts. Implement them through the standard NSUndoManager. Put Undo and Redo in the Edit menu using the standard names.

---

### Workouts
**Path:** Patterns  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/workouts

#### Hero image
![Workouts](images/patterns-workouts-intro@2x.png)
*A sketch of a person running, suggesting exercise. The image is overlaid with rectangular and circular grid lines and is tinted orange to subtly reflect the orange in the original six-color Apple logo.*

#### Summary

People can wear their Apple Watch during many types of workouts, and they might carry their iPhone or iPad during fitness activities like walking, wheelchair pushing, and running. In contrast, people tend to use their larger or more stationary devices like iPad Pro, Mac, and Apple TV to participate in live or recorded workout sessions by themselves or with others.

You can create a workout experience for Apple Watch, iPhone, or iPad that helps people reach their goals by leveraging activity data from the device and using familiar components to display fitness metrics.

#### Best practices

In a watchOS fitness app, use workout sessions to provide useful data and relevant controls. During a fitness app's active workout sessions, watchOS continues to display the app as time passes between wrist raises, so it's important to provide the workout data people are most likely to care about. For example, you might show elapsed or remaining time, calories burned, or distance traveled, and offer relevant controls like lap or interval markers.

Avoid distracting people from a workout with information that's not relevant. For example, people don't need to review the list of workouts you offer or access other parts of your app while they're working out. Many watchOS workout apps use this arrangement:

- Large buttons that control the in-progress session — such as End, Resume, and New — appear on the leftmost screen.
- Metrics and other data appear on a dedicated screen that people can read at a glance.
- If supported, media playback controls appear on the rightmost screen.

Use a distinct visual appearance to indicate an active workout. During a workout, people appreciate being able to recognize an active session at a glance. The metrics page can be a good way to show that a session is active because the values update in real time. In addition to displaying updating values, you can further distinguish the metrics screen by using a unique layout.

Provide workout controls that are easy to find and tap. In addition to making it easy for people to pause, resume, and stop a workout, be sure to provide clear feedback that indicates when a session starts or stops.

Help people understand the health information your app records if sensor data is unavailable during a workout. For example, water may prevent a heart-rate measurement, but your app can still record data like the distance people swam and the number of calories they burned. If your app supports the Swimming or Other workout types, explain the situation using language similar to the system-provided Workout app:

| | Example text from the Workout app |
|-|----------------------------------|
| | GPS is not used during a Pool Swim, and water may prevent a heart-rate measurement, but Apple Watch will still track your calories, laps, and distance using the built-in accelerometer. |
| | In this type of workout, you earn the calorie equivalent of a brisk walk anytime sensor readings are unavailable. |
| | GPS will only provide distance when you do a freestyle stroke. Water might prevent a heart-rate measurement, but calories will still be tracked using the built-in accelerometer. |

Provide a summary at the end of a session. A summary screen confirms that a workout is finished and displays the recorded information. Consider enhancing the summary by including Activity rings, so that people can easily check their current progress.

Discard extremely brief workout sessions. If a session ends a few seconds after it starts, either discard the data automatically or ask people if they want to record the data as a workout.

Make sure text is legible for when people are in motion. When a session requires movement, use large font sizes, high-contrast colors, and arrange text so that the most important information is easy to read.

Use Activity rings correctly. The Activity rings view is an Apple-designed element featuring one or more rings whose colors and meanings match those in the Activity app. Use them only for their documented purpose.

#### Platform considerations

No additional considerations for iOS, iPadOS, or watchOS. Not supported in macOS, tvOS, or visionOS.

---
