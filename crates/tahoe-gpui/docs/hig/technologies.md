# HIG — Technologies

Offline developer reference for all 29 Human Interface Guidelines **Technologies** pages.
Extracted from `https://developer.apple.com/design/human-interface-guidelines/` (light-mode canonical).
Hero images are stored locally under `images/` using the `@2x.png` light-variant filenames.

---

## Section Map

| # | Slug | Title | Coverage | Top-Level Sections |
| --- | --- | --- | --- | --- |
| 1 | `airplay` | AirPlay | Detailed | Best practices, Using AirPlay icons, Referring to AirPlay, Platform considerations, Resources, Change log |
| 2 | `always-on` | Always On | Detailed | Best practices, Platform considerations, Resources, Change log |
| 3 | `app-clips` | App Clips | Detailed | Designing your App Clip, Creating content for an App Clip card, App Clip Codes, Printing guidelines, Legal requirements, Platform considerations, Resources, Change log |
| 4 | `apple-pay` | Apple Pay | Detailed | Offering Apple Pay, Streamlining checkout, Handling errors, Supporting subscriptions, Using Apple Pay buttons, Referring to Apple Pay, Platform considerations, Resources, Change log |
| 5 | `augmented-reality` | Augmented reality | Detailed | Best practices, Providing coaching, Helping people place objects, Designing object interactions, Offering a multiuser experience, Reacting to real-world objects, Communicating with people, Handling interruptions, Suggesting problem resolutions, Icons and badges, Platform considerations, Resources, Change log |
| 6 | `carekit` | CareKit | Detailed | Data and privacy, CareKit views, Notifications, Symbols and branding, Platform considerations, Resources, Change log |
| 7 | `carplay` | CarPlay | Detailed | iPhone interactions, Audio, Layout, Color, Icons and images, Error handling, Platform considerations, Resources, Change log |
| 8 | `game-center` | Game Center | Detailed | Accessing Game Center, Achievements, Leaderboards, Challenges, Multiplayer activities, Platform considerations, Resources, Change log |
| 9 | `generative-ai` | Generative AI | Detailed | Best practices, Transparency, Privacy, Models and datasets, Inputs, Outputs, Continuous improvement, Platform considerations, Resources, Change log |
| 10 | `healthkit` | HealthKit | Detailed | Privacy protection, Activity rings, Apple Health icon, Editorial guidelines, Platform considerations, Resources, Change log |
| 11 | `homekit` | HomeKit | Detailed | Terminology and layout, Setup, Siri interactions, Custom functionality, Using HomeKit icons, Referring to HomeKit, Platform considerations, Resources, Change log |
| 12 | `icloud` | iCloud | Detailed | Best practices, Platform considerations, Resources, Change log |
| 13 | `id-verifier` | ID Verifier | Detailed | Best practices, Platform considerations, Resources, Change log |
| 14 | `imessage-apps-and-stickers` | iMessage apps and stickers | Detailed | Best practices, Specifications, Platform considerations, Resources, Change log |
| 15 | `in-app-purchase` | In-app purchase | Detailed | Best practices, Auto-renewable subscriptions, Platform considerations, Resources, Change log |
| 16 | `live-photos` | Live Photos | Detailed | Best practices, Platform considerations, Resources |
| 17 | `mac-catalyst` | Mac Catalyst | Detailed | Before you start, Choose an idiom, Integrate the Mac experience, Platform considerations, Resources, Change log |
| 18 | `machine-learning` | Machine learning | Detailed | Planning your design, The role of machine learning in your app, Explicit feedback, Implicit feedback, Calibration, Corrections, Mistakes, Multiple options, Confidence, Attribution, Limitations, Platform considerations, Resources, Change log |
| 19 | `maps` | Maps | Detailed | Best practices, Custom information, Place cards, Indoor maps, Platform considerations, Resources, Change log |
| 20 | `nfc` | NFC | Detailed | In-app tag reading, Background tag reading, Platform considerations, Resources |
| 21 | `photo-editing` | Photo editing | Detailed | Best practices, Platform considerations, Resources |
| 22 | `researchkit` | ResearchKit | Detailed | Creating the onboarding experience, Conducting research, Managing personal information and providing encouragement, Platform considerations, Resources, Change log |
| 23 | `shareplay` | SharePlay | Detailed | Best practices, Sharing activities, Platform considerations, Maintaining a shared context, Adjusting a shared context, Resources, Change log |
| 24 | `shazamkit` | ShazamKit | Detailed | Best practices, Platform considerations, Resources |
| 25 | `sign-in-with-apple` | Sign in with Apple | Detailed | Offering Sign in with Apple, Collecting data, Displaying buttons, Platform considerations, Resources, Change log |
| 26 | `siri` | Siri | Detailed | Integrating your app with Siri, System intents, Custom intents, Shortcuts and suggestions, Editorial guidelines, Platform considerations, Resources, Change log |
| 27 | `tap-to-pay-on-iphone` | Tap to Pay on iPhone | Detailed | Enabling Tap to Pay on iPhone, Educating merchants, Checking out, Displaying results, Additional interactions, Platform considerations, Resources, Change log |
| 28 | `voiceover` | VoiceOver | Detailed | Descriptions, Navigation, Platform considerations, Resources, Change log |
| 29 | `wallet` | Wallet | Detailed | Passes, Designing passes, Order tracking, Identity verification, Platform considerations, Specifications, Resources, Change log |

---

## 1. AirPlay

**Path:** `technologies/airplay`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/airplay  
**Hero image:** `images/technologies-AirPlay-intro@2x.png`  

### Best practices

Prefer the system-provided media player. The built-in media player offers a standard set of controls and supports features like chapter navigation, subtitles, closed captioning, and AirPlay streaming. It's also easy to implement, provides a consistent and familiar playback experience across the system, and accommodates the needs of most media apps. Consider designing a custom video player only if the system-provided player doesn't meet your app's needs. For developer guidance, see AVPlayerViewController.

Provide content in the highest possible resolution. Your HTTP Live Streaming (HLS) playlist needs to include the full range of available resolutions so that people can experience your content in the resolution that's appropriate for the device they're using (AVFoundation automatically selects the resolution based on the device). If you don't include a range of resolutions, your content looks low quality when people stream it to a device that can play at higher resolutions. For example, content that looks great on iPhone at 720p will look low quality when people use AirPlay to stream it to a 4K TV.

Stream only the content people expect. Avoid streaming content like background loops and short video experiences that make sense only within the context of the app itself. For developer guidance, see usesExternalPlaybackWhileExternalScreenIsActive.

Support both AirPlay streaming and mirroring. Supporting both features gives people the most flexibility.

Support remote control events. When you do, people can choose actions like play, pause, and fast forward on the lock screen, and through interaction with Siri or HomePod. For developer guidance, see Remote command center events.

Don't stop playback when your app enters the background or when the device locks. For example, people expect the TV show they started streaming from your app to continue while they check their mail or put their device to sleep. In this type of scenario, it's also crucial to avoid automatic mirroring because people don't want to stream other content on their device without explicitly choosing to do so.

Don't interrupt another app's playback unless your app is starting to play immersive content. For example, if your app plays a video when it launches or auto-plays inline videos, play this content on only the local device, while allowing current playback to continue. For developer guidance, see ambient.

Let people use other parts of your app during playback. When AirPlay is active, your app needs to remain functional. If people navigate away from the playback screen, make sure other in-app videos don't begin playing and interrupt the streaming content.

If necessary, provide a custom interface for controlling media playback. If you can't use the system-provided media player, you can create a custom media player that gives people an intuitive way to enter AirPlay. If you need to do this, be sure to provide custom buttons that match the appearance and behavior of the system-provided ones, including distinct visual states that indicate when playback starts, is occurring, or is unavailable. Use only Apple-provided symbols in custom controls that initiate AirPlay, and position the AirPlay icon correctly in your custom player.

### Using AirPlay icons

You can download AirPlay icons in Resources. You have the following options for displaying the AirPlay icon in your app.

#### Black AirPlay icon

Use the black AirPlay icon on white or light backgrounds when other technology icons also appear in black.

#### White AirPlay icon

Use the white AirPlay icon on black or dark backgrounds when other technology icons also appear in white.

#### Custom color AirPlay icon

Use a custom color when other technology icons also appear in the same color.

Position the AirPlay icon consistently with other technology icons. If you display other technology icons within shapes, you can display the AirPlay icon in the same manner.

Don't use the AirPlay icon or name in custom buttons or interactive elements. Use the icon and the name AirPlay only in noninteractive ways.

Pair the icon with the name AirPlay correctly. You can show the name below or beside the icon if you also reference other technologies in this way. Use the same font you use in the rest of your layout. Avoid using the AirPlay icon within text or as a replacement for the name AirPlay.

Emphasize your app over AirPlay. Make references to AirPlay less prominent than your app name or main identity.

### Referring to AirPlay

Use correct capitalization when using the term AirPlay. AirPlay is one word, with an uppercase A and uppercase P, each followed by lowercase letters.

Always use AirPlay as a noun.

Use terms like works with, use, supports, and compatible.

Use the name Apple with the name AirPlay if desired.

Refer to AirPlay if appropriate and to add clarity. If your content is specific to AirPlay, you can use AirPlay to make that clear. You can also refer to AirPlay in technical specifications.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, or visionOS. Not supported in watchOS.

### Resources

#### Related

Apple Design Resources

Apple Trademark List

Guidelines for Using Apple Trademarks and Copyrights

#### Developer documentation

AVFoundation

AVKit

#### Videos

Reaching the Big Screen with AirPlay 2

### Change log

| Date | Changes |
| --- | --- |
| May 2, 2023 | Consolidated guidance into one page. |


---

## 2. Always On

**Path:** `technologies/always-on`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/always-on  
**Hero image:** `images/technologies-always-on-intro@2x.png`  

On devices that include the Always On display, the system can continue to display an app's interface when people suspend their interactions with the device.

In the Always On state, a device can continue to give people useful, glanceable information in a low-power, privacy-preserving way by dimming the display and minimizing onscreen motion. The system can display different items depending on the device.

On iPhone 14 Pro and iPhone 14 Pro Max, the system displays Lock Screen items like Widgets and Live Activities when people set aside their device face up and stop interacting with it.

When people drop their wrist while wearing Apple Watch, the system dims the watch face, continuing to display the interface of the app as long as it's either frontmost or running a background session.

On both devices, the system displays notifications while in Always On, and people can tap the display to exit Always On and resume interactions.

### Best practices

Hide sensitive information. It's crucial to redact personal information that people wouldn't want casual observers to view, like bank balances or health data. You also need to hide personal information that might be visible in a notification.

Keep other types of personal information glanceable when it makes sense. On Apple Watch, for example, people typically appreciate getting pace and heart rate updates while they're working out; on iPhone, people appreciate getting a glanceable update on a flight arrival or a notification when a ride-sharing service arrives. If people don't want any information to be visible, they can turn off Always On.

Keep important content legible and dim nonessential content. You can increase dimming on secondary text, images, and color fills to give more prominence to the information that's important to people. For example, a to-do list app might remove row backgrounds and dim each item's additional details to highlight its title. Also, if you display rich images or large areas of color, consider removing the images and using dimmed colors.

Maintain a consistent layout. Avoid making distracting interface changes when Always On begins or ends and throughout the Always On experience. For example, when Always On begins, prefer transitioning an interactive component to an unavailable appearance — don't just remove it. Within the Always On context, aim to make infrequent, subtle updates to the interface. For example, a sports app might pause granular play-by-play updates while in Always On, only updating the score when it changes. Note that unnecessary changes during Always On can be especially distracting on iPhone, because people often put their device face up on a surface, making motion on the screen visible even when they're not looking directly at it.

Gracefully transition motion to a resting state; don't stop it instantly. Smoothly finishing the current motion helps communicate the transition and avoids making people think that something went wrong.

### Platform considerations

No additional considerations for iOS or watchOS. Not supported in iPadOS, macOS, tvOS, or visionOS.

### Resources

#### Related

Designing for watchOS

#### Developer documentation

Designing your app for the Always On state — watchOS apps

#### Videos

What's new in watchOS 8

Build a workout app for Apple Watch

What's new in SwiftUI

### Change log

| Date | Changes |
| --- | --- |
| September 12, 2023 | Updated intro image artwork. |
| September 23, 2022 | Expanded guidance to cover the Always On display on iPhone 14 Pro and iPhone 14 Pro Max. |


---

## 3. App Clips

**Path:** `technologies/app-clips`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/app-clips  
**Hero image:** `images/technologies-app-clips-intro@2x.png`  

App Clips deliver an experience from your app or game without requiring people to download the full app from the App Store. App Clips focus on a fast solution to a task or contain a demo that showcases the full app or game, and they remain on the device for a limited amount of time while preserving people's privacy.

People discover and launch App Clips in a variety of situations and contexts.

Consider creating an App Clip if your app provides an in-the-moment experience that helps people perform a task over a finite amount of time.

Consider creating an App Clip to let people experience your app or game before committing to a purchase or subscription.

### Designing your App Clip

Allow people to complete a task or a demo in your App Clip. Don't require people to install the full app to experience the entire demo, to complete a task, or to finish a level in a game.

Focus on essential features. Interactions with App Clips are quick and focused. Limit features to what's necessary to accomplish the task at hand.

Don't use App Clips solely for marketing purposes. App Clips need to provide real value and help people accomplish tasks.

Avoid using web views in your App Clip. App Clips use native components and frameworks to offer an app-quality experience.

Design a linear, easy-to-use, and focused user interface. App Clips don't need tab bars, complex navigation, or settings.

On launch, show the most relevant part of your App Clip. Skip unnecessary steps and take people immediately to the part of the App Clip that best fits their context.

Ensure people can use your App Clip immediately. App Clips need to include all required assets, omit splash screens, and never make people wait on launch.

Ensure your App Clip is small. The smaller your App Clip, the faster it will launch on a person's device.

Make the App Clip shareable. When someone shares a link to an App Clip in the Messages app, recipients can launch the App Clip from within the Messages app.

Make it easy to pay for a service or product. Consider supporting Apple Pay to offer express checkout.

Avoid requiring people to create an account before they can benefit from your App Clip.

Provide a familiar, focused experience in your app. When people install the full app, it replaces the App Clip on their device.

#### Preserving privacy

The system imposes limits on App Clips to ensure people's privacy.

Limit the amount of data you store and handle yourself.

Consider offering Sign in with Apple.

Offer a secure way to pay for services or goods that also respects people's privacy.

#### Showcasing your app

People don't manage App Clips themselves, and App Clips don't appear on the Home screen.

Don't compromise the user experience by asking people to install the full app.

Pick the right time to recommend your app.

Recommend your app in a nonintrusive, polite way.

#### Limiting notifications

App Clips provide the option to schedule and receive notifications for up to 8 hours after launch.

Only ask for permission to use notifications for an extended period of time if it's really needed.

Keep notifications focused.

Use notifications to help people complete a task.

#### Creating App Clips for businesses

Use consistent branding.

Consider multiple businesses.

### Creating content for an App Clip card

Be informative. Make sure the image on the App Clip card clearly communicates the features offered by your App Clip.

Prefer photography and graphics. Avoid using a screenshot of your app's user interface.

Avoid using text. Text in the header image isn't localizable.

Adhere to image requirements. Use a 1800x1200 px PNG or JPEG image without transparency.

Use concise copy. An App Clip card requires both a title and a subtitle.

Pick a verb for the action button that best fits your App Clip. Possible verbs are View, Play, or Open.

### App Clip Codes

App Clip Codes are the best way for people to discover your App Clip. Their distinct design is immediately recognizable.

App Clip Codes always use the designs Apple provides and follow size, placement, and printing guidelines.

#### Interacting with App Clip Codes

App Clip Codes come in two variants: scan-only or with an embedded NFC tag (NFC-integrated).

#### Displaying App Clip Codes

When you start designing your App Clip Codes, choose the variant that works best.

Include the App Clip logo when space allows.

Place your App Clip Code on a flat or cylindrical surface only.

Don't create App Clip Codes that are too small.

| Type | Minimum size |
| --- | --- |
| Printed communications | Minimum diameter of 3/4 inch (1.9 cm). |
| Digital communications | Minimum size of 256x256 px. Use a PNG or SVG file. |
| NFC-integrated App Clip Code | The embedded NFC tag needs to be at least 35 mm in diameter. |

#### Using clear messaging

Add clear messaging that informs people how they can use the App Clip Code.

Adhere to Guidelines for Using Apple Trademarks when referring to your App Clip.

#### Customizing your App Clip Code

Always use the generated App Clip Code.

Choose colors with enough contrast that ensure accurate scanning.

### Printing guidelines

App Clip Codes offer the best experience to launch App Clips.

Always test printed App Clip Codes before you distribute them.

Use high-quality, non-textured print materials. Print App Clip Codes on matte finishes.

Use high-resolution images and printer settings.

Use correct color settings when you convert the generated SVG file to a CMYK image.

#### Verifying your printer's calibration

A reliable scanning experience depends on the quality of your printed App Clip Codes.

### Legal requirements

Only the Apple-provided App Clip Codes created in App Store Connect or with the App Clip Code Generator command-line tool are approved for use.

App Clip Codes are approved for use to indicate availability of an App Clip.

You may not use the App Clip Code as part of your own company name or as part of your product name.

### Platform considerations

No additional considerations for iOS or iPadOS. Not supported in macOS, tvOS, visionOS, or watchOS.

### Resources

#### Related

Apple Pay

Sign in with Apple

Guidelines for Using Apple Trademarks and Copyrights

#### Developer documentation

App Clips

App Store Connect

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| June 9, 2025 | Updated guidance to include demo App Clips. |
| May 2, 2023 | Consolidated guidance into one page. |


---

## 4. Apple Pay

**Path:** `technologies/apple-pay`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/apple-pay  
**Hero image:** `images/technologies-Apple-Pay-intro@2x.png`  

People authorize payments and provide shipping and contact information, using credentials that are securely stored on the device.

Apps and websites that accept Apple Pay display it as an available payment option, and include an Apple Pay button in the purchasing flow that people tap to bring up a payment sheet.

The device performs payment authentication in most cases where the device supports Face ID, Touch ID, or Optic ID.

### Offering Apple Pay

Offer Apple Pay on all devices and browsers that support it.

If you also offer other payment methods, offer Apple Pay at the same time.

If you use an Apple Pay button to start the Apple Pay payment process, you must use the Apple-provided API to display it.

Use Apple Pay buttons only to start the Apple Pay payment process and, when appropriate, the Apple Pay set-up process.

Don't hide an Apple Pay button or make it appear unavailable.

Use the Apple Pay mark only to communicate that Apple Pay is accepted.

Inform search engines that Apple Pay is accepted on your website.

### Streamlining checkout

Provide a cohesive checkout experience.

If Apple Pay is available, assume the person wants to use it.

Accelerate single-item purchases with Apple Pay buttons on product detail pages.

Accelerate multi-item purchases with express checkout.

Collect necessary information, like color and size options, before people reach the Apple Pay button.

Collect optional information before checkout begins.

Gather multiple shipping methods and destinations before showing the payment sheet.

Prefer information from Apple Pay.

Avoid requiring account creation prior to purchase.

Report the result of the transaction so that people can view it in the payment sheet.

Display an order confirmation or thank-you page.

#### Customizing the payment sheet

Only present and request essential information.

Display the active coupon or promotional code, or give people a way to enter it.

Let people choose the shipping method in the payment sheet.

Use line items to explain additional charges, discounts, pending costs, add-on donations, recurring, and future payments.

Keep line items short.

Provide a business name after the word Pay on the same line as the total.

If you're not the end merchant, specify both your business name and the end merchant's name in the payment sheet.

Clearly disclose when additional costs may be incurred after payment authorization.

Handle data entry and payment errors gracefully.

#### Displaying a website icon

If your website supports Apple Pay, provide an icon in the following sizes for use in the summary view and the payment sheet.

| @2x | @3x |
| --- | --- |
| 60x60 pt (120x120 px @2x) | 60x60 pt (180x180 px @3x) |

### Handling errors

Provide clear, actionable guidance when problems occur during checkout or payment processing.

#### Data validation

Your app or website can respond to user input when the payment sheet appears.

Avoid forcing compliance with your business logic.

Provide accurate status reporting to the system.

Succinctly and specifically describe the problem when data is invalid or incorrectly formatted.

#### Payment processing

Handle interruptions correctly.

### Supporting subscriptions

Your app or website can use Apple Pay to request authorization for recurring fees.

Clarify subscription details before showing the payment sheet.

Include line items that reiterate billing frequency, discounts, and additional upfront fees.

Clarify the current payment amount in the total line.

Only show the payment sheet when a subscription change results in additional fees.

#### Supporting donations

Approved nonprofits can use Apple Pay to accept donations.

Use a line item to denote a donation.

Streamline checkout by offering predefined donation amounts.

### Using Apple Pay buttons

The system provides several Apple Pay button types and styles you can use in your app or website.

Don't create your own Apple Pay button design or attempt to mimic the system-provided button designs.

#### Button types

Apple provides several types of buttons so you can choose the button type that fits best.

Use the Apple-provided APIs to create Apple Pay buttons.

#### Button styles

You can use the automatic style to let the current system appearance determine the appearance of the Apple Pay buttons.

#### Black

Use on white or light-color backgrounds that provide sufficient contrast.

#### White with outline

Use on white or light-color backgrounds that don't provide sufficient contrast.

#### White

Use on dark-color backgrounds that provide sufficient contrast.

#### Button size and position

Prominently display the Apple Pay button.

Position the Apple Pay button correctly in relation to an Add to Cart button.

Adjust the corner radius to match the appearance of other buttons.

Maintain the minimum button size and margins around the button.

#### Apple Pay mark

Use the Apple Pay mark graphic to show that Apple Pay is an available payment option when showing other payment options in a similar manner.

Use only the artwork provided by Apple, with no alterations other than height.

Maintain a minimum clear space around the mark of 1/10 of its height.

### Referring to Apple Pay

As with all Apple product names, use Apple Pay exactly as shown in Apple Trademark List.

Capitalize Apple Pay in text as it appears in the Apple Trademark list.

Never use the Apple logo to represent the name Apple in text.

Don't translate Apple Pay or any other Apple trademark.

In a payment selection context, you can display a text-only description of Apple Pay only when all payment options have text-only descriptions.

When promoting your app's use of Apple Pay, follow App Store guidelines.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, visionOS, or watchOS. Not supported in tvOS.

### Resources

#### Related

Apple Pay Marketing Guidelines

#### Developer documentation

Apple Pay — PassKit

Apple Pay on the Web

WKInterfacePaymentButton — WatchKit

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| December 16, 2025 | Clarified supported platforms, including web browsers and Apple Vision Pro. |
| June 10, 2024 | Updated links to developer guidance for offering Apple Pay on the web. |
| September 12, 2023 | Updated artwork. |
| May 2, 2023 | Consolidated guidance into one page. |


---

## 5. Augmented reality

**Path:** `technologies/augmented-reality`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/augmented-reality  
**Hero image:** `images/technologies-augmented-reality-intro@2x.png`  

Using the device's camera to present the physical world onscreen live, your app can superimpose three-dimensional virtual objects, creating the illusion that these objects actually exist.

Offer AR features only on capable devices.

Note: The following guidance applies to apps that run in iOS and iPadOS.

### Best practices

Let people use the entire display. Devote as much of the screen as possible to displaying the physical world and your app's virtual objects.

Strive for convincing illusions when placing realistic objects.

Consider how virtual objects with reflective surfaces show the environment.

Use audio and haptics to enhance the immersive experience.

Minimize text in the environment.

If additional information or controls are necessary, consider displaying them in screen space.

Consider using indirect controls when you need to provide persistent controls.

Anticipate that people will use your app in a wide variety of real-world environments.

Be mindful of people's comfort.

If your app encourages people to move, introduce motion gradually.

Be mindful of people's safety.

### Providing coaching

Before people can enjoy an AR experience in your app, they need to move their device in ways that lets ARKit evaluate the surroundings and detect surfaces.

Hide unnecessary app UI while people are using a coaching view.

If necessary, offer a custom coaching experience.

### Helping people place objects

Show people when to locate a surface and place an object.

When people place an object, immediately integrate that object into the AR environment.

Consider guiding people toward offscreen virtual objects.

Avoid trying to precisely align objects with the edges of detected surfaces.

Incorporate plane classification information to inform object placement.

### Designing object interactions

Let people use direct manipulation to interact with objects when possible.

Let people directly interact with virtual objects using standard, familiar gestures.

In general, keep interactions simple.

Respond to gestures within reasonable proximity of interactive virtual objects.

Let people initiate object scaling when it makes sense in your app.

Be wary of potentially conflicting gestures.

Strive for virtual object movement that's consistent with the physics of your app's AR environment.

Explore even more engaging methods of interaction.

### Offering a multiuser experience

When multiple people share your app's AR experience, each participant maps the environment independently.

Consider allowing people occlusion.

When possible, let new participants enter a multiuser AR experience.

### Reacting to real-world objects

You can enhance an AR experience by using known images and objects in the real-world environment to make virtual content appear.

When a detected image first disappears, consider delaying the removal of virtual objects that are attached to it.

Limit the number of reference images in use at one time.

Limit the number of reference images requiring an accurate position.

### Communicating with people

If you must display instructional text, use approachable terminology.

| Do | Don't |
| --- | --- |
| Unable to find a surface. Try moving to the side or repositioning your phone. | Unable to find a plane. Adjust tracking. |
| Tap a location to place the [name of object]. | Tap a plane to anchor an object. |
| Try turning on more lights and moving around. | Insufficient features. |
| Try moving your phone more slowly. | Excessive motion detected. |

In a three-dimensional context, prefer 3D hints.

Make important text readable.

If necessary, provide a way to get more information.

### Handling interruptions

ARKit can't track device position and orientation during an interruption.

Consider using the system-provided coaching view to help people relocalize.

Consider hiding previously placed virtual objects during relocalization.

Minimize interruptions if your app supports both AR and non-AR experiences.

Allow people to cancel relocalization.

Indicate when the front-facing camera is unable to track a face for more than about half a second.

### Suggesting problem resolutions

Let people reset the experience if it doesn't meet their expectations.

Suggest possible fixes if problems occur.

| Problem | Possible suggestion |
| --- | --- |
| Insufficient features detected. | Try turning on more lights and moving around. |
| Excessive motion detected. | Try moving your phone slower. |
| Surface detection takes too long. | Try moving around, turning on more lights, and making sure your phone is pointed at a sufficiently textured surface. |

### Icons and badges

Apps can display an AR icon in controls that launch ARKit-based experiences.

Use the AR glyph as intended.

Maintain minimum clear space around the glyph.

Apps that include collections of products or other objects can use badging to identify specific items that can be viewed in AR.

Use the AR badges as intended and don't alter them.

Prefer the AR badge to the glyph-only badge.

Use badging only when your app contains a mixture of objects that can be viewed in AR and objects that cannot.

Keep badge placement consistent and clear.

Maintain minimum clear space around the badge.

### Platform considerations

No additional considerations for iOS or iPadOS. Not supported in macOS, tvOS, or watchOS.

#### visionOS

With the wearer's permission, you can use ARKit in your visionOS app to detect surfaces in a person's surroundings, use a person's hand and finger positions to inform your custom gestures, support interactions that incorporate nearby physical objects into your immersive experience, and more.

### Resources

#### Related

Playing haptics

Gestures

Apple Design Resources

#### Developer documentation

ARKit

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| May 2, 2023 | Consolidated guidance into one page. |


---

## 6. CareKit

**Path:** `technologies/carekit`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/carekit  
**Hero image:** `images/technologies-CareKit-intro@2x.png`  

To learn more about CareKit, see Research & Care > CareKit.

CareKit 2.0 contains two projects, CareKit UI and CareKit Store. CareKit UI provides a wide variety of prebuilt views you can use to create a custom CareKit app. CareKit Store defines a database scheme that incorporates CareKit entities.

### Data and privacy

Nothing is more important than protecting people's privacy and safeguarding the extremely sensitive data your CareKit app collects and stores.

Provide a coherent privacy policy.

You must receive people's permission before accessing data through these features.

#### HealthKit integration

HealthKit is the central repository for health and fitness data in iOS and watchOS.

Request access to health data only when you need it.

Clarify your app's intent by adding descriptive messages to the standard permission screen.

Manage health data sharing solely through the system's privacy settings.

#### Motion data

If it's useful for treatment and if people give permission, your app can get motion information from the device.

#### Photos

Pictures are a great way to communicate treatment progress.

#### ResearchKit integration

A ResearchKit app lets people participate in important medical research studies.

### CareKit views

CareKit UI provides customizable views organized into three categories — tasks, charts, and contacts.

| Category | Purpose |
| --- | --- |
| Tasks | Present tasks, like taking medication or doing physical therapy. Support logging of patient symptoms and other data. |
| Charts | Display graphical data that can help people understand how their treatment is progressing. |
| Contact views | Display contact information. Support communication through phone, message, and email, and link to a map of the contact's location. |

#### Tasks

A care plan generally presents a set of prescribed actions for people to perform.

Use the simple style for a one-step task.

Use the instructions style when you need to add informative text to a simple task.

Use the log style to help people log events.

Use the checklist style to display a list of actions or steps in a multistep task.

Use the grid style to display a grid of buttons in a multistep task.

Consider using color to reinforce the meaning of task items.

Combine accuracy with simplicity when describing a task and its steps.

Consider supplementing multistep or complex tasks with videos or images.

#### Charts

Chart views let you present data and trends in graphical ways.

Consider highlighting narratives and trends to illustrate progress.

Label chart elements clearly and succinctly.

Use distinct colors.

Consider providing a legend to add clarity.

Clearly denote units of time.

Consolidate large data sets for greater readability.

If necessary, offset data to keep charts proportional.

#### Contact views

A care plan typically includes a care team and other trusted individuals.

Consider using color to categorize care team members.

### Notifications

Notifications can tell people when it's time to take medication or complete a task.

Minimize notifications.

Consider providing a detail view.

### Symbols and branding

CareKit uses a variety of built-in symbols to help people understand what they can do in a care app.

Design a relevant care symbol.

Incorporate refined, unobtrusive branding.

### Platform considerations

No additional considerations for iOS or iPadOS. Not supported in macOS, tvOS, visionOS, or watchOS.

### Resources

#### Related

Research & Care > CareKit

#### Developer documentation

CareKit

Research & Care > Developers

HealthKit

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| May 2, 2023 | Consolidated guidance into one page. |


---

## 7. CarPlay

**Path:** `technologies/carplay`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/carplay  
**Hero image:** `images/technologies-CarPlay-intro@2x.png`  

CarPlay lets people get directions, make calls, send and receive messages, listen to music, and more from their car's built-in display, all while staying focused on the road.

People download CarPlay apps from the App Store and install them on iPhone like any other app. When people connect their iPhone with their vehicle, app icons for installed CarPlay apps appear on the CarPlay Home screen.

CarPlay is designed for drivers to use while they're driving.

To create the interface of your CarPlay app, you use the system-defined templates that are appropriate for the type of app you're developing, such as audio, communication, navigation, or fueling.

### iPhone interactions

CarPlay shows compatible apps from the connected iPhone on the car's built-in display, applying simplified interfaces that are optimized for use while driving.

Eliminate app interactions on iPhone when CarPlay is active.

Never lock people out of CarPlay because the connected iPhone requires input.

Make sure your app works without requiring people to unlock iPhone.

### Audio

In CarPlay, keep in mind that your app coexists with other audio sources.

Let people choose when to start playback.

Start playback as soon as audio has sufficiently loaded.

Display the Now Playing screen when audio is ready to play.

Resume audio playback after an interruption only when it's appropriate.

When necessary, automatically adjust audio levels, but don't change the overall volume.

### Layout

CarPlay supports a wide range of display resolutions with varying pixel densities and aspect ratios.

| Dimensions (pixels) | Aspect ratio |
| --- | --- |
| 800x480 | 5:3 |
| 960x540 | 16:9 |
| 1280x720 | 16:9 |
| 1920x720 | 8:3 |

Provide useful, high-value information in a clean layout that's easy to scan from the driver's seat.

Maintain an overall consistent appearance throughout your app.

Ensure that primary content stands out and feels actionable.

### Color

Color can indicate interactivity, impart vitality, and provide visual continuity.

In general, prefer a limited color palette that coordinates with your app logo.

Avoid using the same color for interactive and noninteractive elements.

Test your app's color scheme under a variety of lighting conditions in an actual car.

Ensure your app looks great in both dark and light environments.

Choose colors that help you communicate effectively with everyone.

### Icons and images

CarPlay supports both landscape and portrait displays and both @2x and @3x scale factors.

Supply high-resolution images with scale factors of @2x and @3x for all CarPlay artwork in your app.

Mirror your iPhone app icon.

Don't use black for your icon's background.

Create your CarPlay app icon in the following sizes: @2x: 120x120, @3x: 180x180.

### Error handling

A CarPlay app needs to handle errors gracefully and report them to people only when absolutely necessary.

Report errors in CarPlay, not on the connected iPhone.

### Platform considerations

No additional considerations for iOS. Not supported in iPadOS, macOS, tvOS, visionOS, or watchOS.

### Resources

#### Related

CarPlay

#### Developer documentation

CarPlay App Programming Guide

#### Videos

Turbocharge your app for CarPlay

### Change log

| Date | Changes |
| --- | --- |
| May 2, 2023 | Consolidated guidance into one page. |


---

## 8. Game Center

**Path:** `technologies/game-center`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/game-center  
**Hero image:** `images/technologies-Game-Center-intro@2x.png`  

Supporting Game Center in your game allows players to discover new games their friends are playing, seamlessly invite friends to play, and see the latest activity from their games across the system.

You can add Game Center into your game using the GameKit framework.

### Accessing Game Center

To provide the best Game Center experience for your players, begin by determining whether the player is signed in to their Game Center account on the system when they launch your game.

#### Integrating the access point

The Game Center access point is an Apple-designed UI element that lets players view their Game Center profile and information without leaving your game.

Display the access point in menu screens.

Avoid placing controls near the access point.

Consider pausing your game while the Game Overlay or dashboard is present.

#### Using custom UI

Your game can include custom links into the Game Overlay or the dashboard.

Use the artwork Game Center provides in custom links.

Use the correct terminology in custom links.

| Term | Incorrect terms | Localization |
| --- | --- | --- |
| Game Center | GameKit, GameCenter, game center | Use the system-provided translation of Game Center |
| Game Center Profile | Profile, Account, Player Info | Use the system-provided translation |
| Achievements | Awards, Trophies, Medals |  |
| Leaderboards | Rankings, Scores, Leaders |  |
| Challenges | Competitions |  |
| Add Friends | Add, Add Profiles, Include Friends |  |

### Achievements

Achievements give players an added incentive to stay engaged with your game.

#### Integrating achievements into your game

Align with Game Center achievement states. Game Center defines four achievement states: locked, in-progress, hidden, and completed.

Determine a display order.

Be succinct when describing achievements.

Give players a sense of progress.

#### Creating achievement images

Design rich, high-quality images that help players feel rewarded.

Create artwork in the appropriate size and format.

### Leaderboards

Leaderboards are a great way to encourage friendly competition within your game.

Choose a leaderboard type. Game Center supports two types of leaderboards: classic and recurring.

Take advantage of leaderboard sets for multiple leaderboards.

Add leaderboard images.

### Challenges

Challenges turn single player activities into multiplayer experiences with friends.

Create engaging challenges.

Avoid creating challenges that track overall progress or personal best scores.

Make it easy to jump into your challenge.

Create high-quality artwork that encourages players to engage with your challenges.

| Attribute | Value |
| --- | --- |
| Format | JPEG, JPG, or PNG |
| Color space | sRGB or P3 |
| Resolution | 72 DPI (minimum) |
| Image size | 1920x1080 pt (3840x2160 px @2x) |

### Multiplayer activities

Game Center supports both real-time and turn-based multiplayer activities.

Use party codes to invite players to multiplayer activities.

Support multiplayer activities through in-game UI.

Provide engaging activity artwork.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, or visionOS.

#### tvOS

Display an optional image at the top of the dashboard.

#### watchOS

Be aware of Game Center support on watchOS. While GameKit features and API are available for watchOS games, keep in mind that there's no system-supported Game Center UI that you can invoke on watchOS.

### Resources

#### Related

Designing for games

Game controls

Apple Design Resources

#### Developer documentation

GameKit

Creating activities for your game

Creating engaging challenges from leaderboards

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| June 9, 2025 | Added guidance for new challenges and multiplayer activities, and considerations for the Apple Games app and Game Overlay. |
| February 2, 2024 | Added links to developer guidance on using the access point and dashboard in a visionOS game. |
| September 12, 2023 | Added artwork for the iOS achievement layout. |
| May 2, 2023 | Consolidated guidance into one page. |


---

## 9. Generative AI

**Path:** `technologies/generative-ai`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/generative-ai  
**Hero image:** `images/technologies-generative-ai-intro@2x.png`  

Generative AI empowers you to enhance your app or game with dynamic content and offer intelligent features that unlock new levels of creativity, connection, and productivity.

Generative artificial intelligence uses machine learning models to create and transform text, images, and other content.

### Best practices

Design your experience responsibly. Responsible AI is the intentional design and development of AI features that considers their direct and indirect impacts on people, systems, and society.

Keep people in control. While AI can manipulate and create content, respect people's agency and ensure they remain in charge of decision making and the overall experience.

Ensure an inclusive experience for all. AI models learn from data and tend to favor the most common information.

Design engaging and useful generative features. Generative AI is a powerful tool, but it's not the right solution for every situation.

Ensure a great experience even when generative features aren't available or people opt not to use them.

### Transparency

Communicate where your app uses AI. Letting people know when and where your app uses AI sets expectations.

Set clear expectations about what your AI-powered feature can and can't do.

### Privacy

Prefer on-device processing. Depending on your needs, you may be able to get great responses using on-device models, which prevent people's information from leaving the device.

Ask permission before using personal information and usage data.

Clearly disclose how your app and its model use and store personal information.

### Models and datasets

Thoughtfully evaluate model capabilities. There are different types of generative models, some of which possess general knowledge, while others are trained for specific tasks.

Be intentional when choosing or creating a dataset.

### Inputs

Guide people on how to use your generative feature.

Raise awareness about and minimize the chance of hallucinations.

Consider consequences and get permission before performing destructive or potentially problematic tasks.

### Outputs

Help people improve requests when blocked or undesirable results occur.

Reduce unexpected and harmful outcomes with thoughtful design and thorough testing.

Strive to avoid replicating copyrighted content.

Factor processing time into your design.

Consider offering alternate versions of results.

### Continuous improvement

Consider ways to improve your model over time.

Let people share feedback on outputs.

Design flexible, adaptable features.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, visionOS, or watchOS.

### Resources

#### Related

Machine learning

Inclusion

Accessibility

Privacy

Loading

Acceptable Use Requirements for the Foundation Models Framework

#### Developer documentation

Apple Intelligence and machine learning

Foundation Models

#### Videos

Explore prompt design & safety for on-device foundation models

Discover machine learning & AI frameworks on Apple platforms

### Change log

| Date | Changes |
| --- | --- |
| June 9, 2025 | New page. |


---

## 10. HealthKit

**Path:** `technologies/healthkit`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/healthkit  
**Hero image:** `images/technologies-HealthKit-intro@2x.png`  

HealthKit is the central repository for health and fitness data in iOS, iPadOS, and watchOS.

When you support HealthKit in your app, you can ask people for permission to access and update their health information.

Important: If your app doesn't provide health and fitness functionality, don't request access to people's private health data.

### Privacy protection

You must request permission to access people's data, and you must take all necessary steps to protect that data.

Provide a coherent privacy policy.

Request access to health data only when you need it.

Clarify your app's intent by adding descriptive messages to the standard permission screen.

Manage health data sharing solely through the system's privacy settings.

### Activity rings

You can enhance your app's health and wellness offerings by displaying the Activity ring element to show people's progress toward their Move, Exercise, and Stand goals.

Use Activity rings for Move, Exercise, and Stand information only.

Use Activity rings to show progress for a single person.

Don't use Activity rings for ornamentation.

Don't use Activity rings for branding.

Maintain Activity ring and background colors.

Maintain Activity ring margins.

Differentiate other ring-like elements from Activity rings.

Provide app-specific information only in Activity notifications.

### Apple Health icon

The Apple Health icon shows that an app works with HealthKit and the Health app.

Use only the Apple-provided icon.

Display the name 'Apple Health' close to the Apple Health icon.

Display the Apple Health icon consistently with other health-related app icons.

Don't use the Apple Health icon as a button.

Don't alter the appearance of the Apple Health icon.

Maintain a minimum clear space around the Apple Health icon of 1/10 of its height.

Don't use the Apple Health icon within text or as a replacement for the terms Health, Apple Health, or HealthKit.

Don't display Health app images or screenshots.

### Editorial guidelines

Refer to the Health app as 'Apple Health' or 'the Apple Health app'.

Don't use the term 'HealthKit'. HealthKit is a developer-facing term that names the framework.

Use correct capitalization when using the term 'Apple Health'.

Use the system-provided translation of 'Health' to avoid confusing people.

### Platform considerations

No additional considerations for iOS, iPadOS, or watchOS. Not supported in macOS, tvOS, or visionOS.

### Resources

#### Related

Works with Apple Health

Activity rings

Apple Design Resources

#### Developer documentation

HealthKit

Protecting user privacy — HealthKit

#### Videos

Meet the HealthKit Medications API

Track workouts with HealthKit on iOS and iPadOS

Explore wellbeing APIs in HealthKit

### Change log

| Date | Changes |
| --- | --- |
| May 2, 2023 | Consolidated guidance into one page. |


---

## 11. HomeKit

**Path:** `technologies/homekit`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/homekit  
**Hero image:** `images/technologies-HomeKit-intro@2x.png`  

In iOS, the Home app also lets people manage and configure accessories.

Your iOS, tvOS, or watchOS app can integrate with HomeKit to provide a custom or accessory-specific experience.

### Terminology and layout

HomeKit models the home as a hierarchy of objects and defines a vocabulary of terms.

Acknowledge the hierarchical model that HomeKit uses.

Make it easy for people to find an accessory's related HomeKit details.

Recognize that people can have more than one home.

Don't present duplicate home settings.

#### Homes

HomeKit uses the term home to represent a physical home, office, or other location.

#### Rooms

A room represents a physical room in a home.

#### Accessories, services, and characteristics

The term accessory represents a physical, connected home accessory.

A controllable feature of an accessory is known as a service.

A characteristic is a controllable attribute of a service.

A service group represents a group of accessory services that someone might want to control as a unit.

#### Actions and scenes

The term action refers to the changing of a service's characteristic.

A scene is a group of actions that control one or more services in one or more accessories.

#### Automations

Automations cause accessories to react to certain situations.

#### Zones

A zone represents an area in the home that contains multiple rooms.

### Setup

Use the system-provided setup flow to give people a familiar experience.

Provide context to explain why you need access to people's Home data.

Don't require people to create an account or supply personal information.

Honor people's setup choices.

Carefully consider how and when to provide a custom accessory setup experience.

#### Help people choose useful names

Suggest service names that suit your accessory.

Check that the names people provide follow HomeKit naming rules.

Help people avoid creating names that include location information.

### Siri interactions

HomeKit supports powerful, hands-free control using voice commands.

Present example voice commands to demonstrate using Siri to control accessories during setup.

After setup, consider teaching people about more complex Siri commands.

Recommend that people create zones and service groups, if they make sense for your accessory.

Offer shortcuts only for accessory-specific functionality that HomeKit doesn't support.

If your app supports both HomeKit and shortcuts, help people understand the difference.

### Custom functionality

Your app is a great place to help people appreciate the unique functionality of your accessory.

Be clear about what people can do in your app and when they might want to use the Home app.

Defer to HomeKit if your database differs from the HomeKit database.

Ask permission to update the HomeKit database when people make changes in your app.

#### Cameras

Your app can display still images or streaming video from a connected HomeKit IP camera.

Don't block camera images.

Show a microphone button only if the camera supports bidirectional audio.

### Using HomeKit icons

Use the HomeKit icon in setup or instructional communications related to HomeKit technology.

Use only Apple-provided icons.

#### Styles

You have several options for displaying the HomeKit icon.

#### Black HomeKit icon

Use the HomeKit icon on white or light backgrounds when other technology icons appear in black.

#### White HomeKit icon

Use the HomeKit icon on black or dark backgrounds when other technology icons appear in white.

#### Custom color HomeKit icon

Use a custom color when other technology icons appear in the same color.

Position the HomeKit icon consistently with other technology icons.

Use the HomeKit icon noninteractively.

Don't use the HomeKit icon within text or as a replacement for the word HomeKit.

Pair the icon with the name HomeKit correctly.

### Referring to HomeKit

Emphasize your app over HomeKit.

Adhere to Apple's trademark guidelines.

#### Referencing HomeKit and the Home app

Use correct capitalization when using the term HomeKit.

Don't use the name HomeKit as a descriptor.

Don't suggest that HomeKit is performing an action or function.

Use the name Apple with the name HomeKit, if desired.

Use the app name Apple Home whenever referring specifically to the app.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, visionOS, or watchOS.

### Resources

#### Related

Apple Design Resources

Guidelines for Using Apple Trademarks and Copyrights

#### Developer documentation

HomeKit

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| May 2, 2023 | Consolidated guidance into one page. |


---

## 12. iCloud

**Path:** `technologies/icloud`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/icloud  
**Hero image:** `images/technologies-iCloud-intro@2x.png`  

iCloud is a service that lets people seamlessly access the content they care about — photos, videos, documents, and more — from any device, without performing explicit synchronization.

A fundamental aspect of iCloud is transparency. People don't need to know where content resides.

### Best practices

Make it easy to use your app with iCloud. People turn on iCloud in Settings and expect apps to work with it automatically.

Avoid asking which documents to keep in iCloud.

Keep content up to date when possible.

Respect iCloud storage space.

Make sure your app behaves appropriately when iCloud is unavailable.

Keep app state information in iCloud.

Warn about the consequences of deleting a document.

Make conflict resolution prompt and easy.

Include iCloud content in search results.

For games, consider saving player progress in iCloud.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, visionOS, or watchOS.

### Resources

#### Related

#### Developer documentation

CloudKit

GameSave

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| June 9, 2025 | Added guidance for synchronizing game data through iCloud. |


---

## 13. ID Verifier

**Path:** `technologies/id-verifier`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/id-verifier  
**Hero image:** `images/technologies-ID-Verifier-Apps-intro@2x.png`  

Beginning in iOS 17, you can integrate ID Verifier into your app, letting iPhone read ISO18013-5 compliant mobile IDs and helping you support in-person ID verification. For example, personnel at a concert venue can use your app on iPhone to verify customers' ages.

Using ID Verifier has advantages for both customers and organizations.

- Customers only present the minimum data needed to prove their age or identity, without handing over their ID card or showing their device.
- Apple provides the key components of the certificate issuance, management, and validation process, simplifying app development and enabling a consistent and trusted ID verification experience.
Depending on the needs of your app, you can use ID Verifier to make the following types of requests:

- Display Only request. Use a Display Only request to display data — such as a person's name or age alongside their photo portrait — within system-provided UI on the requester's iPhone, so the requester can visually confirm the person's identity. When you make a Display Only request, the customer's data remains within the system-provided UI and isn't transmitted to your app. For developer guidance, see MobileDriversLicenseDisplayRequest.
- Data Transfer request. Use a Data Transfer request only when you have a legal verification requirement and you need to store or process information like a person's address or date of birth. You must request an additional entitlement to make a Data Transfer request. To learn more, see Get started with ID Verifier; for developer guidance, see MobileDriversLicenseDataRequest and MobileDriversLicenseRawDataRequest.
### Best practices

Ask only for the data you need. People may lose trust in the experience if you ask for more data than you need to complete the current verification. For example, if you need to ensure that a customer is at least a minimum age, use a request that specifies an age threshold; avoid requesting the customer's current age or birth date. For developer guidance, see ageAtLeast(_:).

If your app qualifies for Apple Business Register, register for ID Verifier to ensure that people can view essential information about your organization when you make a request. Registering for ID Verifier with Apple Business Register lets you provide your official organization name and logo for the system to display on customers' devices as part of the ID verification UI. To learn if your app qualifies and how to register, see Apple Business Register.

Provide a button that initiates the verification process. Use a label like Verify Age in a button that performs a simple age check or Verify Identity for a more detailed identity data request. Avoid including a symbol that specifies a particular type of communication, like NFC or QR codes. Never include the Apple logo in any button label.

| Button type | Example usage |
| --- | --- |
| (Verify Age button) | An app that checks whether people are old enough to attend an event or access a venue, like a concert hall. |
| (Verify Identity button) | An app that verifies whether specific identity information matches expected values, such as name and birth date when picking up a rental car. |

In a Display Only request, help the person using your app provide feedback on the visual confirmation they perform. For example, when the reader displays the customer's portrait, you might provide buttons labeled Matches Person and Doesn't Match Person so your app can receive an approved or rejected value as part of the response.

### Platform considerations

No additional considerations for iOS. Not supported in iPadOS, macOS, tvOS, visionOS, or watchOS.

### Resources

#### Related

Apple Business Register

IDs in Wallet

Identity verification

#### Developer documentation

Adopting the Verifier API in your iPhone app — ProximityReader

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| September 12, 2023 | New page. |


---

## 14. iMessage apps and stickers

**Path:** `technologies/imessage-apps-and-stickers`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/imessage-apps-and-stickers  
**Hero image:** `images/technologies-iMessage-Apps-intro@2x.png`  

An iMessage app can help people share content, collaborate, and even play games with others in a conversation; stickers are images that people can use to decorate a conversation.

An iMessage app or sticker pack is available within the context of a Messages conversation and also in effects in both Messages and FaceTime. You can create an iMessage app or sticker pack as a standalone app or as an app extension within your iOS or iPadOS app. For developer guidance, see Messages and Adding Sticker packs and iMessage apps to the system Stickers app, Messages camera, and FaceTime.

### Best practices

Prefer providing one primary experience in your iMessage app. People are in a conversational flow when they choose your app, so your functionality or content needs to be easy to understand and immediately available. If you want to provide multiple types of functionality or different collections of content, consider creating a separate iMessage app for each one.

Consider surfacing content from your iOS or iPadOS app. For example, your iMessage app could offer app-specific information that people might want to share — such as a shopping list or a trip itinerary — or support a simple, collaborative task, like deciding where to go for a meal or which movie to watch.

Present essential features in the compact view. People can experience your iMessage app in a compact view that appears below the message transcript, or they can expand the view to occupy most of the window. Make sure the most frequently used items are available in the compact view, reserving additional content and features for the expanded view.

In general, let people edit text only in the expanded view. The compact view occupies roughly the same space as the keyboard. To ensure that the iMessage app's content remains visible while people edit, display the keyboard in the expanded view.

Create stickers that are expressive, inclusive, and versatile. Whether your stickers are rich, static images or short animations, make sure that each one remains legible against a wide range of backgrounds and when rotated or scaled. You can also use transparency to help people visually integrate a sticker with text, photos, and other stickers.

For each sticker, provide a localized alternative description. VoiceOver can help people use your sticker pack by speaking a sticker's alternative description.

### Specifications

#### Icon sizes

The icon for an iMessage app or sticker pack can appear in Messages, the App Store, notifications, and Settings. After people install your iMessage app or sticker pack, its icon also appears in the app drawer in the Messages app.

You supply a square-cornered icon for each extension you offer, and the system automatically applies a mask that rounds the corners.

| Usage | @2x (pixels) | @3x (pixels) |
| --- | --- | --- |
| Messages, notifications | 148x110 | - |
| Messages, notifications | 143x100 | - |
| Messages, notifications | 120x90 | 180x135 |
| Messages, notifications | 64x48 | 96x72 |
| Messages, notifications | 54x40 | 81x60 |
| Settings | 58x58 | 87x87 |
| App Store | 1024x1024 | 1024x1024 |

#### Sticker sizes

Messages supports small, regular, and large stickers. Pick the size that works best for your content and prepare all of your stickers at that size; don't mix sizes within a single sticker pack. Messages displays stickers in a grid, organized differently for different sizes.

| Sticker size | @3x dimensions (pixels) |
| --- | --- |
| Small | 300x300 |
| Regular | 408x408 |
| Large | 618x618 |

A sticker file must be 500 KB or smaller in size. For each supported format, the table below provides guidance for using transparency and animation.

| Format | Transparency | Animation |
| --- | --- | --- |
| PNG | 8-bit | No |
| APNG | 8-bit | Yes |
| GIF | Single-color | Yes |
| JPEG | No | No |

### Platform considerations

No additional considerations for iOS or iPadOS. Not supported in macOS, tvOS, visionOS, or watchOS.

### Resources

#### Related

iMessage Apps and Stickers

#### Developer documentation

Messages

Adding Sticker packs and iMessage apps to the system Stickers app, Messages camera, and FaceTime — Messages

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| May 2, 2023 | Consolidated guidance into one page. |


---

## 15. In-app purchase

**Path:** `technologies/in-app-purchase`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/in-app-purchase  
**Hero image:** `images/technologies-IAP-intro@2x.png`  

You can also promote and offer in-app purchases directly through the App Store. For developer guidance, see In-App Purchase.

Using in-app purchase, there are four types of content you can offer:

- Consumable content like lives or gems in a game. After purchase, consumable content depletes as people use it, and people can purchase it again.
- Non-consumable content like premium features in an app. Purchased non-consumable content doesn't expire.
- Auto-renewable subscriptions to virtual content, services, and premium features in your app on an ongoing basis.
- Non-renewing subscriptions to a service or content that lasts for a limited time.
### Best practices

Let people experience your app before making a purchase.

Design an integrated shopping experience.

Use simple, succinct product names and descriptions.

Display the total billing price for each in-app purchase you offer, regardless of type.

Display your store only when people can make payments.

Use the default confirmation sheet.

#### Supporting Family Sharing

People can use Family Sharing to share access to their purchased content — such as auto-renewable subscriptions and non-consumable in-app purchases — with up to five additional family members, across all their Apple devices.

Prominently mention Family Sharing in places where people learn about the content you offer.

Help people understand the benefits of Family Sharing and how to participate.

Aim to customize your in-app messaging so that it makes sense to both purchasers and family members.

#### Providing help with in-app purchases

Sometimes, people need help with a purchase or want to request a refund. To help make this experience convenient, you can present custom UI within your app that provides assistance, offers alternative solutions, and helps people initiate the system-provided refund flow.

Provide help that customers can view before they request a refund.

Use a simple title for the refund action, like "Refund" or "Request a Refund".

Help people find the problematic purchase.

Consider offering alternative solutions.

Make it easy for people to request a refund.

Avoid characterizing or providing guidance on Apple's refund policies.

### Auto-renewable subscriptions

Call attention to subscription benefits during onboarding.

Offer a range of content choices, service levels, and durations.

Consider letting people try your content for free before signing up.

Prompt people to subscribe at relevant times.

Encourage a new subscription only when someone isn't already a subscriber.

#### Making signup effortless

A simple and informative sign-up experience makes it easy for people to act on their interest in your content.

Provide clear, distinguishable subscription options.

Simplify initial signup by asking only for necessary information.

In your tvOS app, help people sign up or authenticate using another device.

Give people more information in your app's sign-up screen.

Clearly describe how a free trial works.

Include a sign-up opportunity in your app's settings.

#### Supporting offer codes

In iOS and iPadOS, subscription offer codes let you use both online and offline channels to give new, existing, and lapsed subscribers free or discounted access to your subscription content.

There are two types of offer codes you can support: one-time use codes and custom codes.

Clearly explain offer details.

Follow guidelines for creating a custom code.

Tell people how to redeem a custom code.

Consider supporting offer redemption within your app.

Supply an engaging and informative promotional image.

Help people benefit from unlocked content as soon as they complete the redemption flow.

#### Helping people manage their subscriptions

Supporting subscription management means people can upgrade, downgrade, or cancel a subscription without leaving your app.

Provide summaries of the customer's subscriptions.

Consider using the system-provided subscription-management UI.

Consider ways to encourage a subscriber to keep their subscription or resubscribe later.

Always make it easy for customers to cancel an auto-renewable subscription.

Consider creating a branded, contextual experience to complement the system-provided management UI.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, or visionOS.

#### watchOS

The sign-up screen in your watchOS app needs to display the same set of information about your subscription options that you display in other versions of your app.

Clearly describe the differences between versions of your app that run on different devices.

Consider using a modal sheet to display the required information.

Make subscription options easy to compare on a small screen.

### Resources

#### Related

In-App Purchase

Offering Subscriptions

App Review Guidelines

#### Developer documentation

In-App Purchase — StoreKit

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| September 12, 2023 | Updated artwork and guidance for redeeming offer codes. |
| November 3, 2022 | Added a guideline for displaying the total billing price for every in-app purchase item and consolidated guidance into one page. |


---

## 16. Live Photos

**Path:** `technologies/live-photos`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/live-photos  
**Hero image:** `images/technologies-Live-Photos-intro@2x.png`  

Live Photos lets people capture favorite memories in a sound- and motion-rich interactive experience that adds vitality to traditional still photos.

When Live Photos is available, the Camera app captures additional content — including audio and extra frames — before and after people take a photo. People press a Live Photo to see it spring to life.

### Best practices

Apply adjustments to all frames. If your app lets people apply effects or adjustments to a Live Photo, make sure those changes are applied to the entire photo. If you don't support this, give people the option of converting it to a still photo.

Keep Live Photo content intact. It's important for people to experience Live Photos in a consistent way that uses the same visual treatment and interaction model across all apps. Don't disassemble a Live Photo and present its frames or audio separately.

Implement a great photo sharing experience. If your app supports photo sharing, let people preview the entire contents of Live Photos before deciding to share. Always offer the option to share Live Photos as traditional photos.

Clearly indicate when a Live Photo is downloading and when the photo is playable. Show a progress indicator during the download process and provide some indication when the download is complete.

Display Live Photos as traditional photos in environments that don't support Live Photos. Don't attempt to replicate the Live Photos experience provided in a supported environment. Instead, show a traditional, still representation of the photo.

Make Live Photos easily distinguishable from still photos. The best way to identify a Live Photo is through a hint of movement. Because there are no built-in Live Photo motion effects, like the one that appears as you swipe through photos in the full-screen browser of Photos app, you need to design and implement custom motion effects.

In cases where movement isn't possible, show a system-provided badge above the photo, either with or without text. Never include a playback button that a viewer can interpret as a video playback button.

Keep badge placement consistent. If you show a badge, put it in the same location on every photo. Typically, a badge looks best in a corner of a photo.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, or tvOS. Not supported in watchOS.

#### visionOS

In visionOS, people can view a Live Photo, but they can't capture one.

### Resources

#### Developer documentation

PHLivePhoto — PhotoKit

LivePhotosKit JS — LivePhotosKit JS

#### Videos


---

## 17. Mac Catalyst

**Path:** `technologies/mac-catalyst`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/mac-catalyst  
**Hero image:** `images/technologies-Mac-Catalyst-intro@2x.png`  

When you use Mac Catalyst to create a Mac version of your iPad app, you give people the opportunity to enjoy the experience in a new environment.

### Before you start

Many iPad apps are great candidates for creating a Mac app built with Mac Catalyst. This is especially true for apps that already work well on iPad and support key iPad features, such as:

- Drag and drop. When you support drag and drop in your iPad app, you also get support for drag and drop in the Mac version.
- Keyboard navigation and shortcuts. Even though a physical keyboard may not always be available on iPad, iPad users appreciate using the keyboard to navigate and keyboard shortcuts to streamline their interactions. On the Mac, people expect apps to offer both keyboard navigation and shortcuts.
- Multitasking. Apps that do a good job scaling the interface to support Split View, Slide Over, and Picture in Picture lay the necessary groundwork to support the extensive window resizability that Mac users expect.
- Multiple windows. By supporting multiple scenes on iPad, you also get support for multiple windows in the macOS version of your app.
Although great iPad apps can provide a solid foundation for creating a Mac app built with Mac Catalyst, some apps rely on frameworks or features that don't exist on a Mac.

Creating a Mac version of your iPad app with Mac Catalyst gives the app automatic support for fundamental macOS features such as pointer interactions, window management, toolbars, rich text interaction, file management, menu bar menus, and app-specific settings.

### Choose an idiom

When you first create your Mac app using Mac Catalyst, Xcode defaults to the 'Scale Interface to Match iPad' setting, or iPad idiom. With this setting, the system ensures that your Mac app appears consistent with the macOS display environment without requiring significant changes to the app's layout. However, text and graphics may appear slightly less detailed because iPadOS views and text scale down to 77% in macOS when you use the iPad idiom.

When your app feels at home on the Mac using the iPad idiom, consider switching to the Mac idiom. With this setting, text and artwork render in more detail, some interface elements and views take on an even more Mac-like appearance, and graphics-intensive apps may see improved performance and lower power consumption.

When you adopt the Mac idiom, thoroughly audit your app's layout, and plan to make changes to it.

Adjust font sizes as needed. With the Mac idiom, text renders at 100% of its configured size, which can appear too large without adjustment. When possible, use text styles and avoid fixed font sizes.

Make sure views and images look good in the Mac version of your app.

Limit your appearance customizations to standard macOS appearance customizations that are the same or similar to those available in iPadOS.

### Integrate the Mac experience

When you use Mac Catalyst to create a Mac version of your iPad app, you need to ensure that your Mac app gives people a rich Mac experience. Regardless of the idiom you choose, it's essential to go beyond simply displaying your iPadOS layout in a macOS window.

#### Navigation

Many iPad and Mac apps organize data in similar ways, but they use different controls and visual indicators to help people understand and navigate through the data.

Typically, iPad apps use split views, tab bars, and page controls to organize their content and features.

If you use a tab bar in your iPad app, consider using a split view with a sidebar or a segmented control.

Make sure people retain access to important tab-bar items in the Mac version of your app.

Offer multiple ways to move between pages. Mac users — especially those who interact using a pointing device or only the keyboard — appreciate Next and Previous buttons in addition to iPad or trackpad gestures.

#### Inputs

Although both iPad and Mac accept user input from a range of devices, touch interactions are the basis for iPadOS conventions. In contrast, keyboard and mouse interactions inform most macOS conventions.

Most iPadOS gestures convert automatically when you create your Mac app using Mac Catalyst.

| iPadOS gesture | Translates to mouse interaction |
| --- | --- |
| Tap | Left or right click |
| Touch and hold | Click and hold |
| Pan | Left click and drag |

#### App icons

Create a macOS version of your app icon. Great macOS app icons showcase the lifelike rendering style that people expect in macOS while maintaining a harmonious experience across all platforms.

#### Layout

To take advantage of the wider Mac screen in ways that give Mac users a great experience, consider updating your layout.

Divide a single column of content and actions into multiple columns.

Use the regular-width and regular-height size classes, and consider reflowing elements in the content area to a side-by-side arrangement as people resize the window.

Present an inspector UI next to the main content instead of using a popover.

Consider moving controls from the main UI of your iPad app to your Mac app's toolbar.

As much as possible, adopt a top-down flow.

Relocate buttons from the side and bottom edges of the screen.

#### Menus

Mac users are familiar with the persistent menu bar and expect to find all of an app's commands in it. In contrast, iPadOS doesn't have a persistent menu bar.

The system automatically converts the context menus in your iPad app to context menus in the macOS version of your app. Mac users tend to expect every object in your app to offer a context menu of relevant actions.

### Platform considerations

No additional considerations for iPadOS or macOS. Not supported in iOS, tvOS, visionOS, or watchOS.

### Resources

#### Related

Designing for macOS

#### Developer documentation

Mac Catalyst — UIKit

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| May 2, 2023 | Consolidated guidance into one page. |


---

## 18. Machine learning

**Path:** `technologies/machine-learning`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/machine-learning  
**Hero image:** `images/technologies-machine-learning-intro@2x.png`  

In addition to providing familiar features like image recognition and content recommendations, your app can use machine learning to forge deep connections with people and help them accomplish more with less effort.

For related guidance on how to use machine learning models to enable intelligent content creation experiences, see Generative AI.

### Planning your design

Machine learning apps use models to perform tasks like recognizing images or finding relationships among numerical data.

Designing the UI and user experience of a machine learning app can be uniquely challenging.

Use the machine learning role you identify to help you define ways your app can receive and display data.

### The role of machine learning in your app

Machine learning systems vary widely, and the ways an app can use machine learning vary widely, too.

#### Critical or complementary

If an app can still work without the feature that machine learning supports, machine learning is complementary to the app; otherwise, it's a critical dependency.

#### Private or public

Machine learning results depend on data. To make good design decisions, you need to know as much as possible about the types of data your app feature needs.

#### Proactive or reactive

A proactive app feature provides results without people requesting it to do so. A reactive app feature provides results when people ask for them or when they take certain actions.

#### Visible or invisible

Apps may use machine learning to support visible or invisible features. People are usually aware of visible app features because such features tend to offer suggestions or choices that people view and interact with.

#### Dynamic or static

All machine learning models can improve, but some improve dynamically, as people interact with the app feature, and others improve offline and affect the feature only when the app updates.

### Explicit feedback

Explicit feedback provides actionable information your app can use to improve the content and experience it presents to people.

Request explicit feedback only when necessary.

Always make providing explicit feedback a voluntary task.

Use simple, direct language to describe each explicit feedback option and its consequences.

Add icons to an option description if it helps people understand it.

Consider offering multiple options when requesting explicit feedback.

Act immediately when you receive explicit feedback and persist the resulting changes.

Consider using explicit feedback to help improve when and where you show results.

### Implicit feedback

Implicit feedback is information that arises as people interact with your app's features.

Always secure people's information.

Help people control their information.

Don't let implicit feedback decrease people's opportunities to explore.

When possible, use multiple feedback signals to improve suggestions and mitigate mistakes.

Consider withholding private or sensitive suggestions.

Prioritize recent feedback.

Use feedback to update predictions on a cadence that matches the person's mental model of the feature.

Be prepared for changes in implicit feedback when you make changes to your app's UI.

Beware of confirmation bias.

### Calibration

Calibration is a process during which people provide information that an app feature needs before it can function.

In general, only use calibration when your feature can't function without that initial information.

Always secure people's information.

Be clear about why you need people's information.

Collect only the most essential information.

Avoid asking people to participate in calibration more than once.

Make calibration quick and easy.

Make sure people know how to perform calibration successfully.

Immediately provide assistance if progress stalls.

Confirm success.

Let people cancel calibration at any time.

Give people a way to update or remove information they provided during calibration.

### Corrections

People use corrections to fix mistakes that apps make.

Give people familiar, easy ways to make corrections.

Provide immediate value when people make a correction.

Let people correct their corrections.

Always balance the benefits of a feature with the effort required to make a correction.

Never rely on corrections to make up for low-quality results.

Learn from corrections when it makes sense.

When possible, use guided corrections instead of freeform corrections.

### Mistakes

It's inevitable that your app will make mistakes. Although people may not expect perfection, mistakes can damage their experience and decrease their trust in your app.

Understand the significance of a mistake's consequences.

Make it easy for people to correct frequent or predictable mistakes.

Continuously update your feature to reflect people's evolving interests and preferences and help avoid mistakes.

When possible, address mistakes without complicating the UI.

Be especially careful to avoid mistakes in proactive features.

As you work on reducing mistakes in one area, always consider the effect your work has on other areas and overall accuracy.

### Multiple options

Depending on the design of your feature, it might work best to present a single result or multiple results from which people can choose.

Prefer diverse options.

In general, avoid providing too many options.

List the most likely option first.

Make options easy to distinguish and choose.

Learn from selections when it makes sense.

### Confidence

Confidence indicates the measure of certainty for a result.

Know what your confidence values mean before you decide how to present them.

In general, translate confidence values into concepts that people already understand.

Consider changing how you present results based on different confidence thresholds.

When you know that confidence values correspond to result quality, you generally want to avoid showing results when confidence is low.

### Attribution

An attribution expresses the underlying basis or rationale for a result, without explaining exactly how a model works.

Consider using attributions to help people distinguish among results.

Avoid being too specific or too general.

Keep attributions factual and based on objective analysis.

In general, avoid technical or statistical jargon.

### Limitations

Every feature has certain limitations to what it can deliver.

Help people establish realistic expectations.

Demonstrate how to get the best results.

Explain how limitations can cause unsatisfactory results.

Consider telling people when limitations are resolved.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, visionOS, or watchOS.

### Resources

#### Related

Generative AI

Privacy

#### Developer documentation

Apple Intelligence and machine learning

Create ML

Core ML

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| October 24, 2023 | Added art to Corrections section. |
| May 2, 2023 | Consolidated guidance into one page. |


---

## 19. Maps

**Path:** `technologies/maps`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/maps  
**Hero image:** `images/technologies-maps-intro@2x.png`  

A map uses a familiar interface that supports much of the same functionality as the system-provided Maps app, such as zooming, panning, and rotation. A map can also include annotations and overlays and show routing information, and you can configure it to use a standard graphical view, a satellite image-based view, or a view that's a hybrid of both.

### Best practices

In general, make your map interactive.

Pick a map emphasis style that suits the needs of your app. There are two emphasis styles: the default style (fully saturated colors) and the muted style (desaturated version of the map).

Help people find places in your map.

Clearly identify elements that people select.

Cluster overlapping points of interest to improve map legibility.

Help people see the Apple logo and legal link.

### Custom information

Use annotations that match the visual style of your app.

If you want to display custom information that's related to standard map features, consider making them independently selectable.

Use overlays to define map areas with a specific relationship to your content.

Make sure there's enough contrast between custom controls and the map.

### Place cards

Place cards display rich place information in your app or website, such as operating hours, phone numbers, addresses, and more.

#### Displaying place cards in a map

You can present a place card that appears directly in your map anytime someone selects a place.

The system defines several place card styles: automatic, callout (full and compact), caption, and sheet.

Consider your map presentation when choosing a style.

Make sure your place card looks great on different devices and window sizes.

Avoid duplicating information.

Keep the location on your map visible when displaying a place card.

#### Adding place cards outside of a map

You can also display place information outside of a map in your app or website.

Use location-related cues in surrounding content to help communicate that people can open a place card.

### Indoor maps

Apps connected with specific venues like shopping malls and stadiums can design custom interactive maps that help people locate and navigate to indoor points of interest.

Adjust map detail based on the zoom level.

Use distinctive styling to differentiate the features of your map.

Offer a floor picker if your venue includes multiple levels.

Include surrounding areas to provide context.

Consider supporting navigation between your venue and nearby transit points.

Limit scrolling outside of your venue.

Design an indoor map that feels like a natural extension of your app.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, or visionOS.

#### watchOS

On Apple Watch, maps are static snapshots of geographic locations. Place a map in your interface at design time and show the appropriate region at runtime. The displayed region isn't interactive; tapping it opens the Maps app on Apple Watch.

Fit the map interface element to the screen.

Show the smallest region that encompasses the points of interest.

### Resources

#### Developer documentation

MapKit

MapKit JS

Indoor Mapping Data Format

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| December 18, 2024 | Added guidance for place cards and included additional artwork. |
| September 12, 2023 | Added artwork. |
| September 23, 2022 | Added guidelines for presenting custom information, refined best practices, and consolidated guidance into one page. |


---

## 20. NFC

**Path:** `technologies/nfc`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/nfc  
**Hero image:** `images/technologies-nfc-intro@2x.png`  

Near-field communication (NFC) allows devices within a few centimeters of each other to exchange information wirelessly.

iOS apps running on supported devices can use NFC scanning to read data from electronic tags attached to real-world objects. For example, a person can scan a toy to connect it with a video game, a shopper can scan an in-store sign to access coupons, or a retail employee can scan products to track inventory.

### In-app tag reading

An app can support single- or multiple-object scanning when the app is active, and display a scanning sheet whenever people are about to scan something.

Don't encourage people to make contact with physical objects. To scan a tag, an iOS device must simply be within close proximity of the tag. Use terms like 'scan' and 'hold near' instead of 'tap' and 'touch' when asking people to scan objects.

Use approachable terminology. Avoid referring to technical, developer-oriented terms like NFC, Core NFC, Near-field communication, and tag. Instead, use friendly, conversational terms that most people will understand.

| Use | Don't use |
| --- | --- |
| Scan the [object name]. | Scan the NFC tag. |
| Hold your iPhone near the [object name] to learn more about it. | To use NFC scanning, tap your phone to the [object]. |

Provide succinct instructional text for the scanning sheet. Provide a complete sentence, in sentence case, with ending punctuation. Identify the object to scan, and revise the text appropriately for subsequent scans.

### Background tag reading

Background tag reading lets people scan tags quickly any time, without needing to first open your app and initiate scanning. On devices that support background tag reading, the system automatically looks for nearby compatible tags whenever the screen is illuminated.

Support both background and in-app tag reading. Your app must still provide an in-app way to scan tags, for people with devices that don't support background tag reading.

### Platform considerations

No additional considerations for iOS or iPadOS. Not supported in macOS, tvOS, visionOS, or watchOS.

### Resources

#### Developer documentation

Core NFC

#### Videos


---

## 21. Photo editing

**Path:** `technologies/photo-editing`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/photo-editing  
**Hero image:** `images/technologies-photo-editing-intro@2x.png`  

Photo-editing extensions let people modify photos and videos within the Photos app by applying filters or making other changes.

Edits are always saved in the Photos app as new files, safely preserving the original versions.

To access a photo editing extension, a photo must be in edit mode. While in edit mode, tapping the extension icon in the toolbar displays an action menu of available editing extensions. Selecting one displays the extension's interface in a modal view containing a top toolbar.

### Best practices

Confirm cancellation of edits. Editing a photo or video can be time consuming. If someone taps the Cancel button, don't immediately discard their changes. Ask them to confirm that they really want to cancel, and inform them that any edits will be lost after cancellation.

Don't provide a custom top toolbar. Your extension loads within a modal view that already includes a toolbar. Providing a second toolbar is confusing and takes space away from the content being edited.

Let people preview edits. It's hard to approve an edit if you can't see what it looks like. Let people see the result of their work before closing your extension and returning to the Photos app.

Use your app icon for your photo editing extension icon. This instills confidence that the extension is in fact provided by your app.

### Platform considerations

No additional considerations for iOS, iPadOS, or macOS. Not supported in tvOS, visionOS, or watchOS.

### Resources

#### Developer documentation

App extensions

PhotoKit

#### Videos


---

## 22. ResearchKit

**Path:** `technologies/researchkit`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/researchkit  
**Hero image:** `images/technologies-ResearchKit-intro@2x.png`  

A research app lets people everywhere participate in important medical research studies.

The ResearchKit framework provides predesigned screens and transitions that make it easy to design and build an engaging custom research app. For developer guidance, see Research & Care > ResearchKit.

These guidelines are for informational purposes only and don't constitute legal advice. Contact an attorney to obtain advice with respect to the development of a research app and any applicable laws.

### Creating the onboarding experience

When opening a research app for the first time, people encounter a series of screens that introduce them to the study, determine their eligibility to participate, request permission to proceed with the study, and, when appropriate, grant access to personal data.

Always display the onboarding screens in the correct order.

#### 1. Introduction

Provide an introduction that informs and provides a call to action. Clearly describe the subject and purpose of your study. Also allow existing participants to quickly log in and continue an in-progress study.

#### 2. Determine eligibility

Determine eligibility as soon as possible. People don't need to move on to the consent section if they're not eligible for the study. Only present eligibility requirements that are necessary for your study.

#### 3. Get informed consent

Make sure participants understand your study before you get their consent. ResearchKit helps you make the consent process concise and friendly.

Break a long consent form into easily digestible sections.

If it makes sense, provide a quiz that tests the participant's understanding.

Get the participant's consent and, if appropriate, some contact information.

#### 4. Request permission to access data

Get permission to access the participant's device or data, and to send notifications. Clearly explain why your research app needs access to location, Health, or other data, and don't request access to data that isn't critical to your study.

### Conducting research

To get input from participants, your study might use surveys, active tasks, or a combination of both.

Create surveys that keep participants engaged. ResearchKit provides many customizable screens you can use in your surveys.

Tell participants how many questions there are and about how long the survey will take.

Use one screen per question.

Show participants their progress in the survey.

Keep the survey as short as possible.

Make active tasks easy to understand.

Describe how to perform the task using clear, simple language.

Make sure participants can tell when the task is complete.

### Managing personal information and providing encouragement

ResearchKit offers a profile screen you can use to let participants manage personal information while they're in your research app.

Use a profile to help participants manage personal data related to your study.

Use a dashboard to show progress and motivate participants to continue.

### Platform considerations

No additional considerations for iOS or iPadOS. Not supported in macOS, tvOS, visionOS, or watchOS.

### Resources

#### Related

Research & Care > ResearchKit

#### Developer documentation

Research & Care > Developers

Protecting user privacy — HealthKit

ResearchKit GitHub project

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| September 12, 2023 | Updated artwork. |


---

## 23. SharePlay

**Path:** `technologies/shareplay`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/shareplay  
**Hero image:** `images/technologies-Share-Play-intro@2x.png`  

SharePlay helps multiple people share activities — like viewing a movie, listening to music, playing a game, or sketching ideas on a whiteboard — while they're in a FaceTime call or Messages conversation.

The system synchronizes app playback on all participating devices to support seamless media and content sharing. In visionOS, SharePlay helps people enjoy these experiences while they're together in the same virtual space.

### Best practices

Let people know that you support SharePlay. People often expect media playback experiences to be shareable, so indicate this capability in your interface. You can use the 'shareplay' SF Symbol to identify the content or experiences in your app that support SharePlay.

If part of your app requires a subscription, consider ways to help nonsubscriber participants quickly join a group activity.

Support Picture in Picture (PiP) when possible.

Use the term SharePlay correctly. You can use SharePlay as a noun or as a verb when describing a direct action in your interface. Avoid using an adjective with SharePlay or changing the term.

### Sharing activities

An activity is an app-defined type of shareable experience.

Briefly describe each activity.

Make it easy to start sharing an activity.

Help people prepare to join a session before displaying the activity.

When possible, defer app tasks that might delay a shared activity.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, or tvOS. Not supported in watchOS.

#### visionOS

People expect most visionOS apps to support SharePlay. While wearing Apple Vision Pro, people choose the Spatial option in FaceTime to share content and activities with others.

In a shared activity, FaceTime can show representations of other participants — called spatial Personas — within each wearer's space.

Choose the spatial Persona template that suits your shared activity. The system provides three spatial Persona templates: side-by-side, surround, and conversational.

The side-by-side template places participants next to each other along a curved line segment, all facing the shared content.

The surround template arranges participants all the way around the shared content in the center.

The conversational template groups participants around a center point, but places your content along the circle, not at its center.

Be prepared to launch directly into your shared activity.

Help people enter a shared activity together, but don't force them.

Smoothly update a shared activity when new participants join.

### Maintaining a shared context

When your shared activity runs in a Full Space, the system helps your app maintain a shared context by using a single coordinate system to arrange your content and all participants.

Make sure everyone views the same state of your app.

Use Spatial Audio to enrich your shared activity.

When possible, let people discover natural, social solutions to confusions or conflicts that might arise during a shared experience.

Help people keep their private and shared content separate.

### Adjusting a shared context

Sometimes, it makes sense to adjust the shared context of a shared activity so each participant can customize their experience.

Let people personalize their experience without changing the experience for others.

Consider when to give each participant a unique view of the shared content.

Make it easy for people to exit and rejoin a shared activity.

### Resources

#### Related

Auto-renewable subscriptions

#### Developer documentation

Group Activities

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| December 5, 2023 | Added artwork for visionOS. |
| June 21, 2023 | Updated to include guidance for visionOS. |
| December 19, 2022 | Clarified guidance for helping nonsubscribers join a group activity. |


---

## 24. ShazamKit

**Path:** `technologies/shazamkit`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/shazamkit  
**Hero image:** `images/technologies-ShazamKit-intro@2x.png`  

ShazamKit supports audio recognition by matching an audio sample against the ShazamKit catalog or a custom audio catalog.

Use cases include enhancing media with graphics, accessibility features (captions/sign language), and synchronizing in-app experiences with virtual content.

Requires microphone permission.

### Best practices

Stop recording as soon as possible.

Let people opt in to storing recognized songs to their iCloud library.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, visionOS, or watchOS.

### Resources

Developer documentation: ShazamKit.

Videos: Explore ShazamKit (WWDC2021).


---

## 25. Sign in with Apple

**Path:** `technologies/sign-in-with-apple`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/sign-in-with-apple  
**Hero image:** `images/technologies-SIWA-intro@2x.png`  

Supporting Sign in with Apple lets people use the Apple Account they already have to sign in or sign up, and skip filling out forms, verifying email addresses, and choosing passwords.

You can offer Sign in with Apple in every version of your app or website across all platforms — including non-Apple platforms.

Sign in with Apple makes it easy for people to authenticate with Face ID, Touch ID, or Optic ID and has two-factor authentication built in.

### Offering Sign in with Apple

Ask people to sign in only in exchange for value.

Delay sign-in as long as possible.

If you require an account, ask people to set it up before offering any sign-in options.

Consider letting people link an existing account to Sign in with Apple.

In a commerce app, wait until after people make a purchase before asking them to create an account.

As soon as Sign in with Apple completes, welcome people to their new account.

Indicate when people are currently signed in.

### Collecting data

Clarify whether the additional data you request is required or just recommended.

Don't ask people to supply a password.

Avoid asking for a personal email address when people supply a private relay address.

Give people a chance to engage with your app before asking for optional data.

Be transparent about the data you collect.

### Displaying buttons

Prominently display a Sign in with Apple button.

#### Using the system-provided buttons

The system provides several variants of the button title.

The following button titles are available for iOS, macOS, tvOS, and the web.

For watchOS, the system provides one title: Sign in.

Depending on the platform, the system provides up to three options: white, white with an outline, and black.

#### White

Use on dark backgrounds.

#### White with outline

Use on white or light-color backgrounds.

#### Black

Use on white or light-color backgrounds.

#### Button size and corner radius

Adjust the corner radius to match other buttons. Default has rounded corners.

| Minimum width | Minimum height | Minimum margin |
| --- | --- | --- |
| 140pt | 30pt | 1/10 of the button's height |

#### Creating a custom Sign in with Apple button

Always make sure people can instantly identify it as a Sign in with Apple button.

Use only logo artwork downloaded from Apple Design Resources.

Titles: use only Sign in with Apple, Sign up with Apple, or Continue with Apple.

Logo and title colors must be black or white.

#### Custom buttons with a logo and text

Choose the format of the logo file based on the height of your button.

| Minimum width | Minimum height | Minimum margin |
| --- | --- | --- |
| 140 pt | 30 pt | 1/10 of the button's height |

#### Custom logo-only buttons

Don't add horizontal padding to a logo-only image.

Use a mask to change the default square shape.

Maintain a minimum margin around the button of at least 1/10 of the button's height.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, visionOS, or watchOS.

### Resources

#### Related

Sign in with Apple button

#### Developer documentation

Authentication Services

Displaying Sign in with Apple buttons on the web

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| September 14, 2022 | Refined guidance on supporting existing accounts. |


---

## 26. Siri

**Path:** `technologies/siri`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/siri  
**Hero image:** `images/technologies-Siri-intro@2x.png`  

When you use SiriKit to define the tasks and actions that your app supports, people can use Siri to perform them even when your app isn’t running. If you’re an accessory maker, you can also help people use Siri to control your accessories by integrating them with HomeKit or AirPlay. Here are some of the ways people can use Siri to interact with your app or accessory:

- Ask Siri to perform a system-defined task that your app supports, like send a message, play a song, or start a workout.
- Run a shortcut, which is a way to accelerate actions your app defines through onscreen interactions or by voice.
- Use the Shortcuts app to adjust what a shortcut does, including combining several actions to perform one multistep shortcut.
- Tap a suggestion to perform a shortcut with your app (Siri can suggest shortcuts that people might want to perform, based on their current context and the information you provide).
- Use Siri to control an accessory that integrates with your app.
Siri works with your products on iPhone, iPad, Mac, Apple Watch, HomePod, and AirPods, so people can use it almost everywhere.

When you make your app’s tasks available through Siri, you have several opportunities to customize the user experience. At a fundamental level, you customize the flow and functionality of the everyday tasks and actions you support to implement your business requirements. To reinforce this functionality throughout the user experience, you can write dialogue that reflects the style and tone of your company’s communications and design custom UI that incorporates your app’s visual style into the Siri interface.

As you approach the job of integrating your app with Siri, assess the actions your app performs and learn how people use your app without voice interaction. Then consider the following steps:

- Identify key tasks in your app that people might want to perform on a regular basis.
- Drive engagement by telling the system about your app’s key tasks and by supporting suggestions.
- For actions that people can perform through voice interaction, design functional conversational flows that feel natural.
- Explore the various ways people might perform your app’s tasks — such as in a hands-free situation — and the devices they might be using, such as Apple Watch or iPad.
### Integrating your app with Siri

Tasks are at the core of your app’s integration with Siri. SiriKit builds on the idea of a person’s intention to perform a task by using the term intent to represent a task an app supports. The communication between your app and Siri is based on the intents — that is, the tasks — your app helps people perform.

SiriKit defines system intents that represent common tasks — such as sending a message, calling a friend, and starting a workout — and groups related intents into domains. A domain is a category of tasks that Siri knows how to talk about, like messaging, calling, and workouts. For a complete list of domains and the actions in each domain that iOS and watchOS support, see System intents.

When possible, take advantage of the intents that SiriKit defines. Using system-provided intents can make your job easier, while still giving you opportunities to customize the experience. However, if your app offers tasks that aren’t represented by system-defined intents — like ordering a meal or shopping for groceries — you can create a custom intent (for guidance, see Custom intents).

#### A closer look at intents

When people use Siri to ask questions and perform actions, Siri does the language processing and semantic analysis needed to turn their requests into intents for your app to handle. The exception is the personal phrase that people create to run a shortcut: When people speak the exact phrase, Siri recognizes it without doing additional processing or analysis.

As a designer, your main job is to present clear, actionable content that helps clarify and streamline the interactions people have with Siri to get things done in your app. Some of these interactions happen while your app and SiriKit communicate about handling the intent, so it’s helpful to be familiar with the related SiriKit terminology. At a high level, your app processes an intent in three phases: resolve, confirm, and handle.

First, your app and SiriKit must agree on what the request means in the resolve phase. You can think of this phase as the time to ask people for everything your app needs and, if necessary, ask for additional information or clarification. For example, if people ask to send a message to Amy, and they have multiple contacts named Amy, a messaging app can have Siri ask which Amy they mean. Details related to an intent, like a message recipient’s name, are known as parameters. In the resolve phase, you can indicate the parameters that are required to complete an action and those that are optional. For developer guidance, see Resolving the Parameters of an Intent.

The second phase — called the confirm phase — happens when you have all the information you need to handle the intent. This phase can give people a chance to make sure they want to complete the task. For example, tasks that have financial impact require confirmation. In addition to asking for a person’s consent, you can present an error during this phase if something will prevent your app from completing the action. For example, if people use an app to order an item for pickup when the pickup location is closed, the app can describe why it can’t complete the action right now. Presenting this error during the confirm phase avoids making people wait until they’re paying for the item to find out that their request has failed. For developer guidance, see Confirming the Details of an Intent.

Third, your app performs the task and tells SiriKit what it actually did in the handle phase. You can provide both visual and textual information that tells people what your app did to handle their request. For example, an app that lets people order coffee might present a receipt that describes the order. Siri can speak or display the information your app provides. For developer guidance, see Handling an Intent.

#### Provide information about actions and support suggestions

Most apps support large numbers of actions, but people tend to perform only a handful of them on a regular basis. When you tell the system about people’s regular actions and describe new ones you think they’ll want to perform in the future, Siri can suggest shortcuts for both types of actions when people are likely to be interested in them.

For example, in an app that’s all about coffee, the most frequent action might be to order a cup of coffee, while less frequent actions might include buying coffee beans or locating a new coffee shop. In this example, the coffee app would share information about the order coffee action so that Siri can suggest a shortcut for this action when people usually want to do it, like weekday mornings. The app could also tell Siri about an action that people haven’t performed yet, but might be interested in — like ordering a new seasonal variation of their favorite coffee — so that Siri might suggest a shortcut for this action.

Siri can use signals like location, time of day, and type of motion (such as walking, running, or driving), to intelligently predict just the right time and place to suggest actions from your app. Depending on the information your app shares and people’s current context, Siri can offer shortcut suggestions on the lock screen, in search results, or on the Siri watch face. Siri can also use some types of information to suggest actions that system apps support, such as using Calendar to add an event shared by your app. Here are some example scenarios.

- Shortly before 7:30 a.m., Siri might suggest the order coffee action to people who use the coffee app every morning.
- After people use a box office–type app to buy tickets to a movie, Siri might remind them to turn on Do Not Disturb shortly before showtime.
- Siri might suggest an automation that starts a workout in a person’s favorite workout app and plays their favorite workout playlist as they enter their usual gym.
- When people enter the airport after a home-bound flight, Siri might suggest they request a ride home from their favorite ride-sharing app.
When you provide information about your actions to the system, people can also use the Shortcuts app to create shortcuts for the system and custom intents you support. For guidance, see Shortcuts and suggestions.

#### Design a great voice experience

A great voice interface helps people feel confident they’ll get the results they want, even when they’re not sure what they can say. Siri supports different voice experiences for system-provided intents and custom intents. With a system intent, Siri does the natural language processing for you, letting people interact with your app in various conversational ways. With a custom intent, your app helps people perform a task that Siri doesn’t know about yet, which results in a different type of support for the voice experience. Custom intents give you additional opportunities to customize conversational dialogue, but also require people to create and speak a precise phrase to start the interaction.

As a designer, you have several ways to enhance both types of conversational experiences and help people specify what they want without engaging in lengthy exchanges.

For system-provided intents, you help Siri communicate with people about the action they want to accomplish by providing essential information and defining any app-specific terminology that might come up during the conversation. You don’t have to write additional dialogue for Siri to speak because Siri already knows about the actions in the system-defined domains and understands many ways that people may talk about them. For example, if you need to confirm the recipient’s name during the resolve phase of a messaging intent, you simply indicate that the required parameter value is missing and Siri says to the sender “Who do you want to send it to?”

Even though you don’t write custom dialogue for system-provided intents, you can enhance the voice experience in other ways. For example, if people ask Siri to “play MyMusicApp” as they enter their gym, you could respond by playing their workout playlist.

When you support a custom intent, people start the action by using their personal invocation phrase; if people don’t speak their phrase precisely, Siri doesn’t initiate the intent. Although you can suggest a memorable phrase for people to use, your principal job is to write clear, direct dialogue, often in the form of follow-up questions, to help people accomplish the action in as few steps as possible.

For example, a coffee app might suggest Order coffee as a phrase people could use to reorder a favorite cup of coffee. In a scenario where people usually use Order coffee to order a cappuccino in various sizes, the coffee app could follow up with custom dialogue that builds on this knowledge, like “What size of cappuccino?” For other types of actions, more open-ended questions can be better at helping people accomplish the task efficiently. For example, if people start a grocery shopping action by saying Add to cart, the grocery shopping app could follow up with “OK, what do you want?”

#### Recognize that people use Siri in different contexts

People can use Siri to get things done while they’re in a car, working out, using apps on a device, or interacting with HomePod. You don’t always know the context in which people are using Siri to perform your app’s actions, so flexibility is key to help people have a great experience no matter what they’re doing.

To communicate with people regardless of their current context, supply information that Siri can provide both vocally and visually. Supporting both voice and screen-based content lets Siri decide which communication method works best for people in their current situation. For example, Siri speaks to people through their AirPods if they say “Hey Siri” while using them.

In voice-only situations, Siri verbally describes information that would have been presented onscreen in other situations. Consider a food-delivery app that requires people to confirm a transaction before completing the order. In a voice-only scenario, Siri may say “Your total is fifteen dollars, and your order will take thirty minutes to arrive at your door. Ready to order?” In contrast, when people can view the cost and delivery information onscreen, Siri might simply say “Ready to order?” When you support custom intents, you’re responsible for supplying the voice-only dialogue that describes these types of onscreen information.

### System intents

SiriKit defines a large number of system intents that represent common tasks people do, such as playing music, sending messages to friends, and managing notes. For system intents, Siri defines the conversational flow, while your app provides the data to complete the interaction.

SiriKit provides the following intents.

| Domain (link to developer guidance) | Intents |
| --- | --- |
| VoIP Calling | Initiate calls. |
| Workouts | Start, pause, resume, end, and cancel workouts. |
| Lists and Notes | Create notes. |
| Search for notes. |
| Create reminders based on a date, time, or location. |
| Media | Search for and play media content, such as video, music, audiobooks, and podcasts. |
| Like or dislike items. |
| Add items to a library or playlist. |
| Messaging | Send messages. |
| Search for messages. |
| Read received messages. |
| Payments | Send payments. |
| Request payments. |
| Car Commands | Activate hazard lights or honk the horn. |
| Lock and unlock the doors. |
| Check the current fuel or power level. |

#### Design responses to system intents

People use Siri for convenience, and they expect a fast response. Your app needs to perform the system intents it supports quickly and accurately so that people have a great experience when they choose your app to get things done.

Whenever possible, complete requests without leaving Siri. If a request must be finished in your app, take people directly to the expected destination. Don’t show intermediary screens or messages that slow down the experience.

When a request has a financial impact, default to the safest and least expensive option. Never deceive people or misrepresent information. For a purchase with multiple pricing levels, don’t default to the most expensive. When people make a payment, don’t charge extra fees without informing them.

When people request media playback from your app, consider providing alternative results if the request is ambiguous. When you display alternative results within the Siri UI, people can easily choose a different piece of content if your first offering isn’t what they want.

On Apple Watch, design a streamlined workflow that requires minimal interaction. Whenever possible, use intelligent defaults instead of asking for input. For example, a music app could respond to a nonspecific request — like “Play music with MyMusicApp” — by playing a favorite playlist. If you must present options to people, offer a small number of relevant choices that reduce the need for additional prompting.

#### Enhance the voice experience for system intents

Help people learn how to use Siri to get things done in your app, and make conversation with Siri feel natural in the context of your brand, by defining app-specific terms and alternative ways people might refer to your app.

Create example requests. When people tap the Help button in the Siri interface, they view a guide that can include example phrases that you supply. Write phrases that demonstrate the easiest and most efficient ways to use Siri with your app. For developer guidance, see Intent Phrases.

Define custom vocabulary that people use with your app. Help Siri learn more about the actions your app performs by defining specific terms people might actually use in requests, like account names, contact names, photo tags, photo album names, ride options, and workout names. Make sure these terms are nongeneric and unique to your app. Never include other app names, terms that are obviously connected with other apps, inappropriate language, or reserved phrases, like Hey Siri. Note that Siri uses the terms you define to help resolve requests, but there’s no guarantee that Siri will recognize them.

Consider defining alternative app names. If people might refer to your app in different ways, it’s a good idea to provide a list of alternative names to help Siri understand what people mean. For example, a UnicornChat app might define the term Unicorn as an alternative app name. Never impersonate other apps by listing their names as alternative names for your app.

#### Design a custom interface for a system intent

If it makes sense in your iOS app, you can supply custom interface elements or a completely custom UI for Siri or Maps to display along with your intent response. A watchOS app can’t provide a custom UI for Siri to display on Apple Watch.

Avoid including extraneous or redundant information. A custom interface lets you bring elements from your app into the Siri interface, but displaying information that isn’t related to the action can distract people. You also want to avoid duplicating information that the system can display in the Siri or Maps interface. For developer guidance, see INParameter.

Make sure people can still perform the action without viewing your custom interface. People can switch to voice-only interaction with Siri at any time, so it’s crucial to help Siri speak the same information that you display in your custom interface.

Use ample margins and padding in your custom interface. Avoid extending content to the edges of your interface unless it’s content that appears to flow naturally offscreen, like a map. In general, provide a margin of 20 points between each edge of your interface and the content. Use the app icon that appears above your interface to guide alignment: content tends to look best when it’s lined up with the center of this icon.

Minimize the height of your interface. The system displays other elements above and below your custom interface, such as the text prompt, the spoken response, and the Siri waveform. Aim for a custom interface height that’s no taller than half the height of the screen, so people can see all your content without scrolling.

Refrain from displaying your app name or icon. The system automatically shows this information, so it’s redundant to include it in your custom interface.

For developer guidance, see Creating an Intents UI Extension.

### Custom intents

If your app lets people perform an everyday task that doesn’t fit into any of the SiriKit domains, you can create a custom intent to represent it (see System intents for a list of domains). You can also use a custom or system intent to support a shortcut, which gives people a quick way to initiate frequently performed actions by speaking a simple phrase or accepting a suggestion from Siri. To learn how to integrate your intents with the system so that people can discover them and add them to Siri, see Shortcuts and suggestions.

#### Custom intent categories and responses

Although your custom intent won’t belong to a SiriKit domain, you’ll need to model it on a system-defined intent category that’s related to your action. SiriKit defines several categories that represent generic tasks, like create, order, share, and search. Because these definitions are in the system, Siri knows how to communicate with people about common actions that are associated with each category — like placing an order or sharing content — in ways that feel natural.

It’s important to choose the category that best represents your action because the category influences the ways Siri speaks about it and the controls people might see in the interface. For example, a coffee app would likely choose the order category to represent its custom order coffee intent, and as a result, Siri can speak default responses that make sense in the context of this action, like “Ready to order?” and “OK. Ordering.” Category choice can have other effects, too: Because the order category includes actions that have financial impact, using this category for the order coffee intent means that people will be asked to authenticate before completing the action.

For several categories, the system defines additional verbs that are related to the category’s default action. You can use these alternative verbs to help ensure that the Siri dialogue and the button titles displayed in the interface align with the way you present your app’s actions. For example, in addition to the default verb order, the order category includes the verbs buy and book.

SiriKit defines the following custom intent categories and associated verbs.

| Category | Default verb | Additional verbs |
| --- | --- | --- |
| Generic | Do | Run, go |
| Information | View | Open |
| Order | Order | Book, buy |
| Start | Start | Navigate |
| Share | Share | Post, send |
| Create | Create | Add |
| Search | Search | Find, filter |
| Download | Download | Get |
| Other | Set | Request, toggle, check in |

SiriKit also defines three response types:

- Confirmation. Confirms that people still want to perform the action.
- Success. Indicates that the action has been initiated.
- Error. Tells people that the action can’t be completed.
In several custom intent categories, SiriKit defines default dialogue for each response type. For example, the default confirmation dialogue for the order category is, “Ready to order?” and the default success dialogue for the share category is, “OK. Shared.”

To customize a response, you create a template that combines dialogue you write with placeholders for relevant information your app can supply while it’s working on the intent. For example, a coffee app might enhance the default order confirmation dialogue by providing custom content that includes a placeholder for the total cost of the order.

Depending on the response type, your custom dialogue is presented before or after the default dialogue. For example, confirmation responses present the default dialogue after any custom dialogue. In the coffee app example, the customized confirmation dialogue would begin with something like, “Your large coffee with cream comes to $2.50” and end with the default dialogue, “Ready to order?”

#### Design a custom intent

If a built-in SiriKit intent represents your action’s purpose, adopt that intent instead of defining a custom intent. For example, if you’d like to offer a shortcut for sending a message, adopt INSendMessageIntent; if you’d like to offer a shortcut for playing media, adopt INPlayMediaIntent. For guidance, see System intents.

If your app’s action requires a custom intent, pick the category that most closely matches the action. A category informs the system about the general function of an intent or shortcut — like order, download, or search — and affects the text and spoken dialogue presented to people when a shortcut is offered by the system or used with Siri. You design the flow of conversation for the custom intents you offer, so it’s essential that you choose a category that corresponds to the meaning of each intent.

> Tip If your action’s primary purpose is to retrieve information or show something to people — like displaying a sports score or the weather — use the information category. Using a different category requires people to make additional taps to get the information.
Design custom intents that accelerate common, useful tasks. Take advantage of the familiarity people have with your app, and make it easier for them to initiate the tasks they perform most often.

Ensure that your intent works well in every scenario. Make it easy for people to run your intent as a shortcut, regardless of how they initiate it. For example, be prepared for people to run it using their voice on devices with and without a screen, from suggestions on the lock screen or the Siri face on Apple Watch, from search, and within a multistep shortcut.

In general, design custom intents for tasks that aren’t overly complex. People benefit the most from intents that reduce the number of actions required to complete a task. Don’t counteract that simplicity by requiring people to engage in a lengthy conversation with your app. You can also reduce the likelihood of user errors by limiting custom intents to clearly defined tasks.

Design your intents to be long-lived. Avoid offering intents that are date-specific or associated with temporary data. For example, it’s not a good idea for a travel app to offer a custom intent for each specific itinerary. A better intent might use follow-up questions to let people get the itinerary for one of their upcoming trips.

Don’t request permission to use Siri. If your app supports only custom intents — and not system intents — you don’t need to get permission to use Siri before letting people create and use voice shortcuts for your intents. Asking for permission can slow people down and could discourage them from using your app’s custom intents.

Support background operation. The best intents support shortcuts that run quickly and don’t pull people out of their current context. Strive to support custom intents that can run in the background without bringing your app to the front. Supporting background operation also ensures that people can complete the task in hands-free and voice-only scenarios.

#### Help people customize their requests

Custom intents can offer follow-up questions that let people do more with a single intent by refining its results on the fly. For example, if you offer an order coffee intent, you can help people get exactly what they want by asking them questions like, “What size?”, “What flavor?”, and “Which location?” Details like size, flavor, and location are parameters your app can define to help people personalize their request.

People supply parameter values to personalize an intent by responding to your follow-up questions or by editing existing values in the Shortcuts app. For example, if you offer an order ground coffee intent that includes a parameter for the grind size, you might supply a follow-up question like, “Which grind?” For people who typically order the coarse grind, you could simplify the interaction by using the value coarse as the default parameter value in a dialogue like, “Do you want coarse-ground coffee?” If people choose a different grind, you can follow up by presenting the full list of options. In voice-only scenarios, Siri speaks your follow-up questions and sends you the responses. When people use the Shortcuts app to edit a parameter value, you receive the new value when they use the associated shortcut. For developer guidance, see Adding User Interactivity with Siri Shortcuts and the Shortcuts App.

Design intents that require as few follow-up questions as possible. Often, an intent can fulfill a request without asking any follow-up questions. Although follow-up questions make intents more flexible, you don’t want to force people into a long interaction. In most cases, it’s best to offer just one or two follow-up questions.

List the smallest number of options possible, and sort the items in a way that makes sense. As with too many follow-up questions, giving people too many options can make completing the task feel onerous. As you determine whether to include an item, consider its complexity as well as its utility. In a food-ordering app, for example, it might be easier for people to parse a list of individual menu items than a list of orders, each of which contains multiple items. After you identify a small number of useful items, consider sorting them by recency, frequency, or popularity.

Make sure each follow-up question is meaningful. Ideally, each follow-up question helps people make an important choice. If options or questions you present are too granular or too similar, the conversation can become repetitive, and people may feel like using your intent is too much work.

Design parameters that are easy for people to understand and use. Aim for parameters that represent simple values or attributes and name them using simple, straightforward terms. For example, a soup-ordering app might define parameters for the type of soup, the serving size, and a delivery location, using names like soup, size, and location. For guidance, see Shortcuts and suggestions.

Ask for confirmation only when necessary. An intent can ask people for confirmation before completing the task or when interpreting an answer to a follow-up question. Apps that support tasks that have financial impact, like an app that helps people place orders, must ask for confirmation before completing an order. For tasks that don’t have financial impact, asking for confirmation can feel like too much extra work and can sometimes discourage people from completing their request. In all cases, avoid asking for confirmation more than once.

Support follow-up questions when it makes sense. For example, an app that helps people order food might offer options for pickup or delivery, but ask for a specific location only after people choose the delivery option.

Prioritize the options you offer based on the context in which people run your shortcut. For example, if people use your shortcut to order an item for pickup, offer pickup locations that are currently close by. Offering options that adapt to the context in which your shortcut is run can help people avoid creating separate shortcuts for specific options.

Consider adjusting the parameter values you offer when people set up your shortcut. When you indicate that a parameter has dynamic options, you can enhance the shortcut setup experience in two ways:

- You can find and present parameter values that are relevant to the context people are in while they’re setting up the shortcut. For example, if people use the Shortcuts app to choose a value for a store-location parameter, the parameter can dynamically generate a list of stores that are currently closest to the device.
- You can present a comprehensive list of parameter values. When people set up a shortcut, having an extensive list of parameter values can help them create the shortcut they want. In contrast, when people use a shortcut to accelerate an action, they generally prefer the convenience of having a shorter list of choices.
For developer guidance, see the storeLocation parameter in the intent definition file of the Soup Chef: Accelerating App Interactions with Shortcuts sample code project.

#### Enhance the voice experience for custom intents

Aim to create conversational interactions. You can customize what Siri says throughout the voice experience, including the handling of follow-up questions. Try writing a script and acting it out with another person to see how well your dialogue works in a face-to-face exchange. Experiencing custom dialogue in this way can help you find places where the interaction doesn’t feel natural.

Help people understand errors and failures. The system provides some default error descriptions, but it’s best to enhance error responses so that they’re specific to the current situation. For example, if chicken noodle soup is sold out, a soup app can respond with a custom error like, “Sorry, we’re out of chicken noodle soup” instead of “Sorry, we can’t complete your order.”

Strive for engaging voice responses. Remember that people may perform your app’s tasks from their HomePod, using “Hey Siri” with their AirPods, or through CarPlay without looking at a screen. In these cases, the voice response needs to convey the same essential information that the visual elements display to ensure that people can get what they need no matter how they interact with Siri.

Create voice responses that are concise, descriptive, and effective in voice-driven scenarios. As with a shortcut title, an effective custom spoken response clearly conveys what’s happening as the shortcut runs. If you ask follow-up questions, be sure to customize the default dialogue for clarity. For example, “Which soup?” is clearer than “Which one?”

Avoid unnecessary repetition. People tend to run voice shortcuts frequently, so they may hear the same prompt multiple times when answering follow-up questions or dealing with errors. Use the context of the current conversation to remove as many details from the prompts as possible. Avoid including unnecessary words or attempts at humor, because both can become irritating over time.

Help conversations with Siri feel natural. People interact with Siri in a variety of ways, like choosing a list item by saying “the second one,” or, in the case of a soup-ordering app, saying “large” or “small” instead of “bowl” or “cup.” You can make people’s Siri interactions feel more natural when you give the system alternative terms and phrases that work as app-specific synonyms (like using “bowl” as a synonym for “large”). Also consider enhancing clarity by providing alternative dialogue options for Siri to speak. For example, the soup app might present a list of onscreen menu options like “1 clam chowder,” or “1 clam chowder and 1 tomato,” but speak these options as “Which order? The one with clam chowder only or the one that includes tomato?”

Exclude your app name. The system provides verbal and visual attribution for your app when responding to people. Including your appʼs name in a verbal response is redundant and may make the experience of interacting with Siri feel less natural. Siri speaks your app’s name less frequently when people have used a shortcut several times, because it isn’t necessary to keep reminding them which app is responding.

Don’t attempt to mimic or manipulate Siri. Never impersonate Siri, attempt to reproduce the functionality that Siri provides, or provide a response that appears to come from Apple.

Be appropriate and respect parental controls. Never present offensive or demeaning content. Keep in mind that many families use parental controls to restrict explicit content and content that’s based on specific rating levels.

Avoid using personal pronouns. Create content that’s inclusive of all people.

Consider letting people view more options in your app. If the list of options doesn’t include the items people need, you might want to include an item that lets people open your app to see more. In the list, you could use copy like, “See more in App Name,” and in spoken dialogue, you might encourage people to say, “More options.”

Keep responses device-independent. People may use Siri to interact with your app via Apple Watch, HomePod, iPad, iPhone, or CarPlay. If you must provide device-specific wording, make sure it accurately reflects the person’s current device.

Don’t advertise. Don’t include advertisements, marketing, or in-app purchase sales pitches in your intent content.

### Shortcuts and suggestions

When you support shortcuts, people have a variety of ways to discover and interact with the custom and system intents your app provides. For example:

- Siri can suggest a shortcut for an action people have performed at least once by offering it in search results, on the lock screen, and in the Shortcuts app.
- Your app can supply a shortcut for an action that people haven’t done yet but might want to do in the future, so that the Shortcuts app can suggest it or it can appear on the Siri watch face.
- People can use the Shortcuts app to view all their shortcuts and even combine actions from different apps into multistep shortcuts.
- People can also use the Shortcuts app to automate a shortcut by defining the conditions that can run it, like time of day or current location.
The Shortcuts app is also available in macOS 12 and later and in watchOS 7 and later. For developer guidance, see SiriKit.

> Developer note The Add to Siri method for adding shortcuts is no longer supported. See App Shortcuts for ways to integrate your app with Siri and the system.
#### Make app actions widely available

Donating information about the actions your app supports helps the system offer them to people in various ways, such as:

- In search results
- Throughout the Shortcuts app
- On the lock screen as a Siri Suggestion
- Within the Now Playing view (for recently played media content)
- During Wind Down
Donations also power Automation Suggestions in the Shortcut app’s Gallery, making it easy for people to set up automations for hands-free interactions with your app.

You can also tell the system about shortcuts for actions people haven’t taken yet or make a shortcut available on the Siri watch face (for guidance, see Suggest Shortcuts people might want to add to Siri and Display shortcuts on the Siri watch face). For developer guidance, see Donating Shortcuts.

Make a donation every time people perform the action. When you donate a shortcut each time people perform the associated action, you help the system more accurately predict the best time and place to offer the shortcut.

Only donate actions that people actually perform. For example, a coffee-ordering app donates the Order coffee shortcut every time people order coffee, but not when people do something else, like browse the menu. Similarly, a media app donates information about a song — like its title and album — only when people are actually listening to it. (For developer guidance, see Improving Siri Media Interactions and App Selection.)

Remove donations for actions that require corresponding data. If information required by a donated action no longer exists, your app needs to delete the donation so the shortcut isn’t suggested anymore. For example, if people delete a contact in a messaging app, the app needs to delete donations for messaging that contact. When people create a shortcut themselves, only they can delete it. For developer guidance, see Deleting Donated Shortcuts.

If your app handles reservations, consider donating them to the system. These items — like ticketed events, travel itineraries, or reservations for restaurants, flights, or movies — automatically appear as suggestions in Calendar or Maps. When you donate a reservation, it can appear on the lock screen with a suggestion to check in with your app or as a reminder that uses current traffic conditions to recommend when people should leave. For developer guidance, see Donating Reservations.

#### Suggest Shortcuts people might want to add to Siri

If your app supports an action that people haven’t performed yet but might find useful, you can provide a suggested shortcut to the system so that people can discover it. For example, if people use a coffee-ordering app to order their daily coffee but not to order a holiday special, the app might still want to give them a way to do this with an Order holiday coffee shortcut.

Suggested shortcuts appear in both the Gallery and the shortcut editor in the Shortcuts app. For developer guidance, see Offering Actions in the Shortcuts App.

#### Display shortcuts on the Siri watch face

On Apple Watch, people can run shortcuts in several ways. For example, people can ask Siri, tap a shortcut complication on a watch face, or use the Shortcuts app available in watchOS 7 and later. You can also make shortcuts available on the Siri watch face.

To have a shortcut appear on the Siri watch face, you define a relevant shortcut by including information like the time of day at which your shortcut is relevant and how the shortcut can display on the Siri watch face. The information you supply lets the Siri watch face intelligently display your shortcut to people when they’re in the appropriate context.

For developer guidance, see Defining Relevant Shortcuts for the Siri Watch Face.

#### Create shortcut titles and subtitles

Shortcut titles and subtitles appear when the system suggests them. In Siri Suggestions on iPhone and Apple Watch, a shortcut can also display an image.

Be concise but descriptive. An effective title conveys what happens when the shortcut runs. A subtitle can provide additional detail that supplements — but doesn’t duplicate — the title.

Start titles with a verb and use sentence-style capitalization without punctuation. Think of a shortcut title as a brief instruction.

|  | Example title |
| --- | --- |
|  | Order my favorite coffee |
|  | Large latte |
|  | Show today’s forecast |
|  | Weather forecast |

Lead with important information. Long titles and subtitles may be truncated in certain contexts, depending on the device’s screen size.

Exclude your app name. The system already identifies the app associated with a shortcut.

Localize titles and subtitles. Providing content in multiple languages ensures an equally great experience for people everywhere.

Consider providing a custom image for a more engaging suggestion. For example, the shortcut for Order my favorite coffee could show a cup of the customer’s favorite coffee. Create an image that measures:

- 60x60 pt (180x180 px @ 3x) to display in an iOS app
- 34x34 pt (68x68 px @2x) to display on the Siri watch face on the 44mm Apple Watch (watchOS scales down the image for smaller watches)
#### Provide default phrases for shortcuts

Your app provides default phrases for shortcuts during setup. People can personalize these phrases when adding your shortcuts to Siri.

Keep phrases short and memorable. Bear in mind that people must speak your phrase verbatim, so long or confusing phrases may result in mistakes and frustration. Two- and three-word phrases tend to work best. More words can be harder for people to remember, and phrases that are too long will get truncated.

Make sure the phrases you suggest are accurate and specific. Phrases like Reorder coffee or Order my usual coffee clearly describe what the shortcut does, which makes it easier for people to remember the phrase later. Also make sure that your suggested phrases are specific to each shortcut’s scope. For example, Watch baseball is clearer and more memorable than Watch sports. It’s also important to avoid implying that people can vary a shortcut’s invocation phrase to get a different result. For example, people might interpret a phrase like Order a large clam chowder to mean that your shortcut will give them what they want if they substitute “small” for “large” and “lobster bisque” for “clam chowder.”

Don’t commandeer core Siri commands. For example, never suggest a phrase like Call 911 or include the text Hey Siri.

#### Make shortcuts customizable

When you define a parameter for each detail your app needs to perform an intent, people can customize the shortcut by editing these details in the Shortcuts app.

To show people which details they can edit and how their edits affect the action, you provide a parameter summary. A parameter summary succinctly describes the action by using the parameters in a sentence that begins with a verb. For example, a customizable Order coffee shortcut could display a parameter summary like “Order quantity coffee” where quantity and coffee are the parameters that people can edit. Here’s an example of how the Order coffee shortcut might look after people supply values for the quantity and coffee parameters.

Provide a parameter summary for each custom intent you support. At minimum, include in your parameter summary all parameters your intent requires and any parameters that receive values from other apps or actions. The summary doesn’t have to include optional parameters or parameters that people aren’t likely to edit; if you omit parameters like these from the summary, people can still access them in the Show More section.

Craft a short parameter summary that’s clearly related to your intent’s title. When the intent title and the parameter summary are similar, it’s easy for people to recognize the action regardless of where they view it. Aim to use the same words in the summary and the title — in particular, it’s helpful to begin both phrases with the same verb. For example, if your intent title is “Search encyclopedia,” a good parameter summary could be “Search encyclopedia for search term.”

Aim for a parameter summary that reads like a sentence. Use sentence-style capitalization, but don’t include ending punctuation. When possible, avoid punctuation entirely. Punctuation within a summary — especially colons, semicolons, and parentheses — can make the summary hard to read and understand.

Provide multiple parameter summaries when necessary. If your action includes a parameter that has a parent-child relationship with other parameters, you can provide multiple variants of the summary based on the current value of the parent parameter. For example, if your order coffee shortcut lets people specify whether they want to pick up their order or have it delivered, your parameter summary can reflect the current choice. In this scenario, create one parameter summary that helps people pick a store location and another summary that helps them pick a delivery address. Be sure to use a consistent grammatical structure and parameter order in all variants of the summary that you create.

Provide output parameters for information that people can use in a multistep shortcut. For example, an order coffee action might provide output that includes the estimated delivery time and the cost of the order. With this information, people could create a multistep shortcut that messages a friend about the delivery time and logs the transaction in their favorite budgeting app.

Consider defining an input parameter. When you define an input parameter for an action, the action can automatically receive output from a preceding action in a multistep shortcut. For example, if your action applies a filter to the image it receives in an image parameter, you might designate image as the input parameter so that it automatically accepts images from other actions. You configure an input parameter in your intent definition file (shown in Adding User Interactivity with Siri Shortcuts and the Shortcuts App).

Help people distinguish among different variations of the same action. For example, an app that offers a send message action might use a contact photo to help people visually distinguish the various messages they send. To do this, choose the parameter that’s most identifiable to people and designate it as the key parameter (shown in Adding User Interactivity with Siri Shortcuts and the Shortcuts App). Be sure to provide an image for the key parameter every time you donate the action (for developer guidance, see INImage).

Avoid providing multiple actions that perform the same basic task. For example, instead of providing an action that adds text to a note and a different action that adds an image, consider providing a single action that lets people add both types of content. Providing a few high-level actions can make it easier for people to understand what the actions do when they’re combined in a multistep shortcut.

For developer guidance, see Shortcut-Related UI.

### Editorial guidelines

Don’t refer to Siri using pronouns like “she,” “him,” or “her.” Ideally, just use the word Siri. For example, After you add a shortcut to Siri, you can run the shortcut anytime by asking Siri.

Use correct capitalization and punctuation when using the term Hey Siri. Hey Siri is two words, italicized or in quotes, with an uppercase H and uppercase S. Do not follow the term with an ellipsis.

|  | Example text |
| --- | --- |
|  | Say Hey Siri to activate Siri. |
|  | Say “Hey Siri” to activate Siri. |
|  | Say Hey Siri… to activate Siri. |
|  | Say “hey Siri” to activate Siri. |

In a localized context, translate only the word Hey in the phrase Hey Siri. As an Apple trademark, Siri is never translated. Here is a list of acceptable translations for the phrase Hey Siri:

| Locale code | Hey Siri translation | Locale code | Hey Siri translation |
| --- | --- | --- | --- |
| ar_AE | يا Siri | fr_CA | Dis Siri |
| ar_SA | يا Siri | fr_CH | Dis Siri |
| da_DK | Hej Siri | fr_FR | Dis Siri |
| de_AT | Hey Siri | it_CH | Ehi Siri |
| de_CH | Hey Siri | it_IT | Ehi Siri |
| de_DE | Hey Siri | ja_JP | Hey Siri |
| en_AU | Hey Siri | ko_KR | Siri야 |
| en_CA | Hey Siri | ms_MY | Hai Siri |
| en_GB | Hey Siri | nb_NO | Hei Siri |
| en_IE | Hey Siri | nl_BE | Hé, Siri |
| en_IN | Hey Siri | nl_NL | Hé Siri |
| en_NZ | Hey Siri | no_NO | Hei Siri |
| en_SG | Hey Siri | pt_BR | E aí Siri |
| en_US | Hey Siri | ru_RU | привет Siri |
| en_ZA | Hey Siri | sv_SE | Hej Siri |
| es_CL | Oye Siri | th_TH | หวัดดี Siri |
| es_ES | Oye Siri | tr_TR | Hey Siri |
| es_MX | Oye Siri | zh_CN | 嘿Siri |
| es_US | Oye Siri | zh_HK | 喂 Siri |
| fi_FI | Hei Siri | zh_TW | 嘿 Siri |
| fr_BE | Dis Siri |  |  |

#### Referring to Shortcuts

When referring to the Shortcuts feature or app, always typeset with a capital S and make sure that Shortcuts is plural. For example, MyApp integrates with Shortcuts to provide a quick way to get things with just a tap or by asking Siri.

When referring to individual shortcuts (that is, not the feature or the Shortcuts app), use lowercase. For example, Run a shortcut by asking Siri or tapping a suggestion on the Lock Screen.

Use the right terminology when describing how people can use Shortcuts in your app. People run shortcuts by asking Siri, so your wording needs to be very similar to phrases like Run a shortcut by asking Siri or Run the shortcut by asking Siri with your personalized phrase (localized as appropriate). Avoid using phrases like add voice shortcuts, make a voice command, create a voice prompt, or any other variation. Instead, consider a phrase like Add a shortcut to Siri to run with your voice (localized as appropriate).

To encourage people to create or use shortcuts in ways other than voice — like automations, Home Screen shortcuts, and other methods — use a phrase that doesn’t specify a particular method, like For quick access, add to Shortcuts.

> Note Use translations of your app name and the word Shortcuts — but not Siri — when referring to them in a localized context.
#### Referring to Apple products

Adhere to Apple’s trademark guidelines. Apple trademarks can’t appear in your app name or images. In text, use Apple product names exactly as shown on the Apple Trademark List.

- Use Apple product names in singular form only; don’t make Apple product names possessive.
- Don’t translate Apple, Siri, or any other Apple trademark.
- Don’t use category descriptors. For example, say iPad, not tablet.
- Don’t indicate any kind of sponsorship, partnership, or endorsement from Apple.
- Attribute Apple, Siri, and all other Apple trademarks with the correct credit lines wherever legal information appears within your app.
- Refer to Apple devices and operating systems only in technical specifications or compatibility descriptions.
See Guidelines for Using Apple Trademarks.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, visionOS, or watchOS.

### Resources

#### Related

App Shortcuts

Design for intelligence

Guidelines for using Apple trademarks and copyrights

#### Developer documentation

SiriKit

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| June 5, 2023 | Removed Add to Siri guidance. Added references to the new App Shortcuts page. |
| May 2, 2023 | Consolidated guidance into one page. |


---

## 27. Tap to Pay on iPhone

**Path:** `technologies/tap-to-pay-on-iphone`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/tap-to-pay-on-iphone  
**Hero image:** `images/technologies-TapToPay-intro@2x.png`  

When you support Tap to Pay on iPhone in your iOS payment app, you help merchants present a consistent and trusted payment experience to their customers.

Before you can integrate Tap to Pay on iPhone into your iOS app, you need to work with a supported payment service provider (PSP), request the entitlement, and use ProximityReader APIs.

> Note If your PSP offers an SDK that supplies user interfaces for experiences like showing a tap result, see the documentation the PSP provides.
### Enabling Tap to Pay on iPhone

Help merchants accept Tap to Pay on iPhone terms and conditions before they begin interacting with their customers.

Present Tap to Pay on iPhone terms and conditions only to an administrative user.

If necessary, help merchants make sure their device is up to date.

### Educating merchants

Provide a tutorial that describes the supported payment types and shows how to use Tap to Pay on iPhone.

You can build your app's tutorial using Apple-approved assets or use the ProximityReaderDiscovery API.

If you design your own tutorial, make sure it shows how to launch a checkout flow, help a customer position their card, and handle PIN entry.

### Checking out

Provide Tap to Pay on iPhone as a checkout option whether the feature is enabled or not.

Avoid making merchants wait to use Tap to Pay on iPhone — prepare as soon as your app starts.

Make sure the Tap to Pay on iPhone checkout option is available even if configuration is continuing in the background.

For the label of the button, use 'Tap to Pay on iPhone' or, if space is constrained, 'Tap to Pay'.

Design your Tap to Pay on iPhone button to match the other buttons in your app.

Determine the final amount before merchants initiate the Tap to Pay on iPhone experience.

### Displaying results

Start processing a transaction as soon as possible.

Display a progress indicator while payment is authorizing.

Clearly display the result of a transaction, whether it's declined or successful.

Help merchants complete the checkout flow when a payment can't complete with Tap to Pay on iPhone.

### Additional interactions

Use a generic label in a button that opens Tap to Pay on iPhone screen when there is no transaction amount.

If your app supports an independent loyalty card transaction, distinguish this flow from a payment-acceptance flow.

### Platform considerations

No additional considerations for iOS. Not supported in iPadOS, macOS, tvOS, visionOS, or watchOS.

### Resources

#### Related

Tap to Pay on iPhone Marketing guidelines

#### Developer documentation

Adding support for Tap to Pay on iPhone to your app — ProximityReader

### Change log

| Date | Changes |
| --- | --- |
| January 17, 2024 | Updated merchant education guidance. |
| May 7, 2024 | Updated to include guidance on enabling the feature and educating merchants. |
| March 3, 2023 | Enhanced guidance for educating merchants and improving their experience. |
| September 14, 2022 | Refined guidance on preparing Tap to Pay on iPhone. |


---

## 28. VoiceOver

**Path:** `technologies/voiceover`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/voiceover  
**Hero image:** `images/technologies-VoiceOver-intro@2x.png`  

VoiceOver is a screen reader that lets people experience your app's interface without needing to see the screen.

By supporting VoiceOver, you help people who are blind or have low vision access information in your app and navigate its interface and content when they can't see the display.

VoiceOver is supported in apps and games built for Apple platforms. It's also supported in apps and games developed in Unity using Apple's Unity plug-ins.

### Descriptions

You inform VoiceOver about your app's content by providing alternative text that explains your app's interface and the content it displays.

Provide alternative labels for all key interface elements. VoiceOver uses alternative labels (not visible onscreen) to audibly describe your app's interface.

Describe meaningful images. If you don't describe key images in your app's content, people can't use VoiceOver to fully experience them.

Make charts and other infographics fully accessible. Provide a concise description of each infographic that explains what it conveys.

Exclude purely decorative images from VoiceOver. It's unnecessary to describe images that are decorative and don't convey useful or actionable information.

### Navigation

Use titles and headings to help people navigate your information hierarchy.

Specify how elements are grouped, ordered, or linked. Proximity, alignment, and other visible contextual cues help sighted people perceive relationships between elements.

VoiceOver reads elements in the same order people read content in their active language and locale.

Inform VoiceOver when visible content or layout changes occur.

Support the VoiceOver rotor when possible. People can use the VoiceOver rotor to navigate a document or webpage by headings, links, and other content types.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, tvOS, or watchOS.

#### visionOS

Be mindful that custom gestures aren't always accessible. When VoiceOver is turned on in visionOS, apps and games that define custom gestures don't receive hand input by default.

### Resources

#### Related

Accessibility

Inclusion

#### Developer documentation

Accessibility

VoiceOver

Supporting VoiceOver in your app

#### Videos

Writing Great Accessibility Labels (WWDC2019)

Tailor the VoiceOver experience in your data-rich apps (WWDC2021)

VoiceOver efficiency with custom rotors (WWDC2020)

### Change log

| Date | Changes |
| --- | --- |
| March 7, 2025 | New page. |


---

## 29. Wallet

**Path:** `technologies/wallet`  
**URL:** https://developer.apple.com/design/human-interface-guidelines/wallet  
**Hero image:** `images/technologies-Wallet-intro@2x.png`  

People use their cards and passes in Wallet to make Apple Pay purchases, track their orders, confirm their identity, and streamline activities like boarding a plane, attending a concert, or receiving a discount.

When you integrate Apple Wallet into your app, you can create custom passes and present them the moment people need them, securely verify an individual's identity so they can access personal content, and offer detailed receipts and tracking information where it's most convenient. For developer guidance, see Wallet.

### Passes

Offer to add new passes to Wallet. When people do something that results in a new pass — like checking into a flight, purchasing an event ticket, or registering for a store reward program — you can present system-provided UI that helps them add the pass to Wallet with one tap (for developer guidance, see addPasses(_:withCompletionHandler:)). If people want to review a pass before adding it, you can display a custom view that displays the pass and provides an Add to Apple Wallet button; for developer guidance, see PKAddPassesViewController.

Help people add a pass that they created outside of your app. If people create a pass using your website or another device, suggest adding it to Wallet the next time they open your app. If people decline your suggestion, don't ask them again.

Add related passes as a group. If your app generates multiple passes, like boarding passes for a multi-connection flight, add all passes at the same time so people don't have to add each one individually. If people can receive a group of passes from your website — such as a set of tickets for an event — bundle them together so that people can download all of them at one time. For developer guidance, see Distributing and updating a pass.

Display an Add to Apple Wallet button to let people add an existing pass that isn't already in Wallet. If people previously declined your suggestion to add a pass to Wallet — or if they removed the pass — a button makes it easy to add it if they change their minds. You can display an Add to Apple Wallet button wherever the corresponding pass information appears in your app. For developer guidance, see PKAddPassButton. You can also display an Add to Apple Wallet badge in an email or on a webpage; for guidance, see Add to Apple Wallet guidelines.

Let people jump from your app to their pass in Wallet. Wherever your app displays information about a pass that exists in Wallet, you can offer a link that opens it directly. Label the link something like "View in Wallet."

Tell the system when your pass expires. Wallet automatically hides expired passes to reduce crowding, while also providing a button that lets people revisit them. To help ensure the system hides passes appropriately, set the expiration date, relevant date, and voided properties of each pass correctly; for developer guidance, see Pass.

Always get people's permission before deleting a pass from Wallet. For example, you could include an in-app setting that lets people specify whether they want to delete passes manually or have them removed automatically. If necessary, you can display an alert before deleting a pass.

Help the system suggest a pass when it's contextually relevant. Ideally, passes automatically appear when they're needed so people don't have to manually locate them. When you supply information about when and where your pass is relevant, the system can display a link to it on the Lock Screen when people are most likely to want it. For example, a gym membership card could appear on the Lock Screen as people enter the gym. For developer guidance, see Showing a Pass on the Lock Screen. Starting in iOS 18 and watchOS 11, the system starts a Live Activity for poster event ticket style passes when they're relevant.

Lock screen notification

Live Activity

Update passes as needed. Physical passes don't typically change, but a digital pass can reflect updates to events. An airline boarding pass, for example, can automatically update to display flight delays and gate changes.

Use change messages only for updates to time-critical information. A change message interrupts people's current workflow, so it's essential to send one only when you make an update they need to know about. For example, people need to know when there's a gate change in a boarding pass, but they don't need to know when a customer service phone number changes. Never use a change message for marketing or other noncritical communication. Change messages are available on a per-field basis; for developer guidance, see Adding a Web Service to Update Passes.

### Designing passes

Wallet uses a consistent design aesthetic to strengthen familiarity and build trust. Instead of merely replicating the appearance of a physical item, design a clean, simple pass that looks at home in Wallet.

Design a pass that looks great and works well on all devices. Passes can look different on different devices. For example, when a pass appears on Apple Watch, it doesn't display all the images it displays when it appears on iPhone (for guidance, see Passes for Apple Watch). Don't put essential information in elements that might be unavailable on certain devices. Also, don't add padding to images; for example, watchOS crops white space from some images.

Avoid using device-specific language. You can't predict the device people will use to view your pass, so don't write text that might not make sense on a particular device. For example, text that tells people to "slide to view" content doesn't make sense when it appears on Apple Watch.

Make your pass instantly identifiable. Using color — especially a color that's linked to your brand — can help people recognize your pass as soon as they see it. Make sure that pass content remains comfortably readable against the background you choose.

Keep the front of a pass uncluttered so people can get important information at a glance. Show essential information — like an event date or account balance — in the top-right area of the pass so people can still see it when the pass is collapsed in Wallet. Use the rest of the pass front to provide important information; consider putting extra information on the back of a pass (iOS) or in a details screen (watchOS).

Prefer an NFC-compatible pass. People appreciate having a contactless pass, because it means that they can just hold their device near a reader. If you support both NFC and a barcode or QR code, the code appears on the back of the pass (in iOS) or in the details screen (in watchOS). In iOS, you can display a QR code or barcode on the front of your pass if necessary for your design.

Reduce image sizes for optimal performance. People can receive passes via email or a webpage. To make downloads as fast as possible, use the smallest image files that still look great.

Provide an icon that represents your company or brand. The system includes your icon when displaying information about a relevant pass on the Lock Screen. Mail also uses the icon to represent your pass in an email message. You can use your app icon or design an icon for this purpose.

#### Pass styles

The system defines several pass styles for categories like boarding pass, coupon, store card, and event ticket. Pass styles specify the appearance and layout of content in your pass, and the information that the system needs to suggest your pass when it's relevant (for guidance, see Passes).

Although each pass style is different, all styles display information using the basic layout areas shown below:

All passes display a logo image, and some can display additional images in other areas depending on the pass style. To display information in the layout areas, use the following PassFields.

| Field | Layout area | Use to provide… |
| --- | --- | --- |
| Header | Essential | Critical information that needs to remain visible when the pass is collapsed in Wallet. |
| Primary | Primary | Important information that helps people use the pass. |
| Secondary and auxiliary | Secondary and auxiliary | Useful information that people might not need every time they use the pass. |
| Back | Not shown in diagram | Supplemental details that don't need to be on the pass front. |

In general, a pass can have up to three header fields, one primary field, up to four secondary fields, and up to four auxiliary fields. Depending on the amount of content you display in each field, some fields may not be visible.

Display text only in pass fields. Don't embed text in images — it's not accessible and not all images are displayed on all devices — and avoid using custom fonts that might make text hard to read.

#### Boarding passes

Use the boarding pass style for train tickets, airline boarding passes, and other types of transit passes. Typically, each pass corresponds to a single trip with a specific starting and ending point.

A boarding pass can display logo and footer images, and it can have up to two primary fields and up to five auxiliary fields.

#### Coupons

Use the coupon style for coupons, special offers, and other discounts. A coupon can display logo and strip images, and it can have up to four secondary and auxiliary fields, all displayed on one row.

#### Store cards

Use the store card style for store loyalty cards, discount cards, points cards, and gift cards. If an account related to a store card carries a balance, the pass usually shows the current balance.

A store card can display logo and strip images, and it can have up to four secondary and auxiliary fields, all displayed on one row.

#### Event tickets

Use the event ticket pass style to give people entry into events like concerts, movies, plays, and sporting events. Typically, each pass corresponds to a specific event, but you can also use a single pass for several events, as with a season ticket.

An event ticket can display logo, strip, background, or thumbnail images. However, if you supply a strip image, don't include a background or thumbnail image. You can also include an extra row of up to four auxiliary fields. For developer guidance, see the row property of PassFields.AuxiliaryFields.

In iOS 18 and later, the system defines an additional style for contactless event tickets called poster event ticket. Poster event tickets offer a rich visual experience that prominently features the event artwork, provides easy access to additional event information, and integrates with system apps like Weather and Maps.

A poster event ticket displays an event logo and background image, and can optionally display a separate ticket issuer or event company logo. The system uses metadata about your event to structure ticket information and suggest relevant actions. You must provide a required set of metadata in SemanticTags for all poster event tickets, and an additional set of required metadata depending on the event type — general, sports, or live performance. You can also add optional metadata to further enhance your ticket.

The system uses the metadata that you provide to generate a Maps shortcut to the venue directions and an event guide below the ticket when in the Wallet app.

Create a vibrant and engaging background. As the centerpiece of a poster event ticket, your background image serves as a visual representation of the event.

Ensure sufficient contrast so that ticket information is easy to read. By default, the system applies a gradient in the header and a blur effect in the footer.

Consider using the additional information tile for extra event details.

Continue to support event tickets for earlier versions of iOS.

#### Generic passes

Use the generic style for a type of pass that doesn't fit into the other categories, such as a gym membership card or coat-check claim ticket. A generic pass can display logo and thumbnail images, and it can have up to four secondary and auxiliary fields, all displayed on one row.

#### Passes for Apple Watch

On Apple Watch, Wallet displays passes in a scrolling carousel of cards. People can add your pass to their Apple Watch even if you don't create a watch-specific app, so it's important to understand how your pass can look on the device.

People can tap a pass on their Apple Watch to reveal a details screen that displays additional information in a scroll view.

### Order tracking

When you support order tracking, Wallet can display information about an order a customer placed through your app or website.

Make it easy for people to add an order to Wallet.

Make information about an order available immediately after people place it.

Provide fulfillment information as soon as it's available, and keep the status up to date.

Supply a high-resolution logo image that uses a nontransparent background.

Supply distinct, high-resolution product images that use nontransparent backgrounds.

In general, keep text brief.

Use clear, approachable language, and localize the text you provide.

#### Displaying order and fulfillment details

An order gives people ways to contact the merchant and displays details about their Apple Pay purchase.

Provide a link to an area where people manage their order.

Clearly describe each item so people can verify that their order contains everything they expect.

Supply a prioritized list of your apps that might be installed on the device.

Avoid sending duplicate notifications.

Make it easy for customers to contact the merchant.

Help people track their order.

Keep the fulfillment screen centered on order tracking.

Keep customers informed through relevant fulfillment status descriptions.

Be direct and thorough when describing an Issue or Canceled status.

### Identity verification

On iPhone running iOS 16 and later, people can store an ID card in Wallet, and later allow an app or App Clip to access information on the card to verify their identity.

To help you offer a consistent experience that people can trust, Apple provides a Verify with Wallet button.

Present a Wallet verification option only when the device supports it.

Ask for identity information only at the precise moment you need it.

Clearly and succinctly describe the reason you need the information you're requesting.

Ask only for the data you actually need.

Clearly indicate whether you will keep the data and — if you need to keep it — specify how long you'll do so.

Choose the system-provided verification button that matches your use case and the visual design of your app.

### Platform considerations

No additional considerations for iOS, iPadOS, macOS, visionOS, or watchOS. Not supported in tvOS.

### Specifications

#### Pass image dimensions

As you design images for your wallet passes, create PNG files and use the following values for guidance.

| Image | Supported pass styles | Filename | Dimensions (pt) |
| --- | --- | --- | --- |
| Logo | Boarding pass, coupon, store card, event ticket, generic pass | logo.png | Any, up to 160x50 |
| Primary logo | Poster event ticket | primaryLogo.png | Any, up to 126x30 |
| Secondary logo | Poster event ticket | secondaryLogo.png | Any, up to 135x12 |
| Icon | All | icon.png | 38x38 |
| Background | Event ticket, poster event ticket | background.png (event ticket), artwork.png (poster event ticket) | 180x220 (event ticket), 358x448 (poster event ticket) |
| Strip | Coupon, store card, event ticket | strip.png | 375x144 (coupon, store card), 375x98 (event ticket) |
| Footer | Boarding pass | footer.png | Any, up to 286x15 |
| Thumbnail | Event ticket, generic pass | thumbnail.png | 90x90 |

### Resources

#### Related

Apple Pay

ID Verifier

#### Developer documentation

FinanceKitUI

FinanceKit

PassKit (Apple Pay and Wallet)

Wallet Passes

Wallet Orders

#### Videos

### Change log

| Date | Changes |
| --- | --- |
| January 17, 2025 | Added specifications for pass image dimensions. |
| December 18, 2024 | Added guidance for the poster event ticket style. |
| September 12, 2023 | Added guidance for helping people add orders to Wallet. |
| February 20, 2023 | Enhanced guidance for presenting order-tracking information and added artwork. |
| November 30, 2022 | Added guidance to include a carrier name in status information for a shipping fulfillment. |
| September 14, 2022 | Added guidelines for using Verify with Wallet, updated guidance on providing shipping status values and descriptions, and consolidated guidance into one page. |
