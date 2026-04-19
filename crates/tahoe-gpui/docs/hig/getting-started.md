## Getting started

Start shaping an app or game that feels native on every Apple platform you support.

### Section overview

The HIG's getting started area is the platform-orientation layer: it explains what makes each platform feel distinct before you design detailed screens and flows.

### Section map

| Page | Coverage | Canonical URL |
|---|---|---|
| Designing for games | Detailed | https://developer.apple.com/design/human-interface-guidelines/designing-for-games |
| Designing for iOS | Detailed | https://developer.apple.com/design/human-interface-guidelines/designing-for-ios |
| Designing for iPadOS | Detailed | https://developer.apple.com/design/human-interface-guidelines/designing-for-ipados |
| Designing for macOS | Detailed | https://developer.apple.com/design/human-interface-guidelines/designing-for-macos |
| Designing for tvOS | Detailed | https://developer.apple.com/design/human-interface-guidelines/designing-for-tvos |
| Designing for visionOS | Detailed | https://developer.apple.com/design/human-interface-guidelines/designing-for-visionos |
| Designing for watchOS | Detailed | https://developer.apple.com/design/human-interface-guidelines/designing-for-watchos |

### Detailed pages

### Designing for games
**Path:** Getting started  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/designing-for-games

#### Hero image

![Designing for games](images/platforms-games-intro%402x.png)
*A stylized representation of a game controller shown on top of a grid. The image is overlaid with rectangular and circular grid lines and is tinted green to subtly reflect the green in the original six-color Apple logo.*

#### Summary

As you create or adapt a game for Apple platforms, learn how to integrate the fundamental platform characteristics and patterns that help your game feel at home on all Apple devices. To learn what makes each platform unique, see Designing for iOS, Designing for iPadOS, Designing for macOS, Designing for tvOS, Designing for visionOS, and Designing for watchOS. For developer guidance, see Games Pathway.

#### Jump into gameplay

Let people play as soon as installation completes. You don't want a player's first experience with your game to be waiting for a lengthy download. Include as much playable content as you can in your game's initial installation while keeping the download time to 30 minutes or less. Download additional content in the background. For guidance, see Loading.

Provide great default settings. People appreciate being able to start playing without first having to change a lot of settings. Use information about a player's device to choose the best defaults for your game, such as the device resolution that makes your graphics look great, automatic recognition of paired accessories and game controllers, and the player's accessibility settings. Also, make sure your game supports the platform's most common interaction methods. For guidance, see Settings.

Teach through play. Players often learn better when they discover new information and mechanics in the context of your game's world, so it can work well to integrate configuration and onboarding flows into a playable tutorial that engages people quickly and helps them feel successful right away. If you also have a written tutorial, consider offering it as a resource players can refer to when they have questions instead of making it a prerequisite for gameplay. For guidance, see Onboarding.

Defer requests until the right time. You don't want to bombard people with too many requests before they start playing, but if your game uses certain sensors on an Apple device or personalizes gameplay by accessing data like hand-tracking, you must first get the player's permission (for guidance, see Privacy). To help people understand why you're making such a request, integrate it into the scenario that requires the data. For example, you could ask permission to track a player's hands between an initial cutscene and the first time they can use their hands to control the action. Also, make sure people spend quality time with your game before you ask them for a rating or review (for guidance, see Ratings and reviews).

#### Look stunning on every display

Make sure text is always legible. When game text is hard to read, people can struggle to follow the narrative, understand important instructions and information, and stay engaged in the experience. To keep text comfortably legible on each device, ensure that it contrasts well with the background and uses at least the recommended minimum text size in each platform. For guidance, see Typography; for developer guidance, see Adapting your game interface for smaller screens.

| Platform | Default text size | Minimum text size |
|---|---|---|
| iOS, iPadOS | 17 pt | 11 pt |
| macOS | 13 pt | 10 pt |
| tvOS | 29 pt | 23 pt |
| visionOS | 17 pt | 12 pt |
| watchOS | 16 pt | 12 pt |

Make sure buttons are always easy to use. Buttons that are too small or too close together can frustrate players and make gameplay less fun. Each platform defines a recommended minimum button size based on its default interaction method. For example, buttons in iOS must be at least 44x44 pt to accommodate touch interaction. For guidance, see Buttons.

| Platform | Default button size | Minimum button size |
|---|---|---|
| iOS, iPadOS | 44x44 pt | 28x28 pt |
| macOS | 28x28 pt | 20x20 pt |
| tvOS | 66x66 pt | 56x56 pt |
| visionOS | 60x60 pt | 28x28 pt |
| watchOS | 44x44 pt | 28x28 pt |

Prefer resolution-independent textures and graphics. If creating resolution-independent assets isn't possible, match the resolution of your game to the resolution of the device. In visionOS, prefer vector-based art that can continue to look good when the system dynamically scales it as people view it from different distances and angles. For guidance, see Images.

Integrate device features into your layout. For example, a device may have rounded corners or a camera housing that can affect parts of your interface. To help your game look at home on each device, accommodate such features during layout, relying on platform-provided safe areas when possible (for developer guidance, see Positioning content relative to the safe area). For guidance, see Layout; for templates that include safe-area guides, see Apple Design Resources.

Make sure in-game menus adapt to different aspect ratios. Games need to look good and behave well at various aspect ratios, such as 16:10, 19.5:9, and 4:3. In particular, in-game menus need to remain legible and easy to use on every device — and, if you support them, in both orientations on iPhone and iPad — without obscuring other content. To help ensure your in-game menus render correctly, consider using dynamic layouts that rely on relative constraints to adjust to different contexts. Avoid fixed layouts as much as possible, and aim to create a custom, device-specific layout only when necessary. For guidance, see In-game menus.

Design for the full-screen experience. People often enjoy playing a game in a distraction-free, full-screen context. In macOS, iOS, and iPadOS, full-screen mode lets people hide other apps and parts of the system UI; in visionOS, a game running in a Full Space can completely surround people, transporting them somewhere else. For guidance, see Going full screen.

#### Enable intuitive interactions

Support each platform's default interaction method. For example, people generally use touch to play games on iPhone; on a Mac, players tend to expect keyboard and mouse or trackpad support; and in a visionOS game, people expect to use their eyes and hands while making indirect and direct gestures. As you work to ensure that your game supports each platform's default interaction method, pay special attention to control sizing and menu behavior, especially when bringing your game from a pointer-based context to a touch-based one.

| Platform | Default interaction methods | Additional interaction methods |
|---|---|---|
| iOS | Touch | Game controller |
| iPadOS | Touch | Game controller, keyboard, mouse, trackpad, Apple Pencil |
| macOS | Keyboard, mouse, trackpad | Game controller |
| tvOS | Remote | Game controller, keyboard, mouse, trackpad |
| visionOS | Touch | Game controller, keyboard, mouse, trackpad, spatial game controller |
| watchOS | Touch | – |

Support physical game controllers, while also giving people alternatives. Every platform except watchOS supports physical game controllers. Although the presence of a game controller makes it straightforward to port controls from an existing game and handle complex control mappings, recognize that not every player can use a physical game controller. To make your game available to as many players as possible, also offer alternative ways to interact with your game. For guidance, see Physical controllers.

Offer touch-based game controls that embrace the touchscreen experience on iPhone and iPad. In iOS and iPadOS, your game can allow players to interact directly with game elements, and to control the game using virtual controls that appear on top of your game content. For design guidance, see Touch controls.

#### Welcome everyone

Prioritize perceivability. Make sure people can perceive your game's content whether they use sight, hearing, or touch. For example, avoid relying solely on color to convey an important detail, or providing a cutscene that doesn't include descriptive subtitles or offer other ways to read the content. For specific guidance, see:

- Text sizes
- Color and effects
- Motion
- Interactions
- Buttons

Help players personalize their experience. Players have a variety of preferences and abilities that influence their interactions with your game. Because there's no universal configuration that suits everyone, give players the ability to customize parameters like type size, game control mapping, motion intensity, and sound balance. You can take advantage of built-in Apple accessibility technologies to support accessibility personalizations, whether you're using system frameworks or Unity plug-ins.

Give players the tools they need to represent themselves. If your game encourages players to create avatars or supply names or descriptions, support the spectrum of self-identity and provide options that represent as many human characteristics as possible.

Avoid stereotypes in your stories and characters. Ask yourself whether you're depicting game characters and scenarios in a way that perpetuates real-life stereotypes. For example, does your game depict enemies as having a certain race, gender, or cultural heritage? Review your game to uncover and remove biases and stereotypes and — if references to real-life cultures and languages are necessary — be sure they're respectful.

#### Adopt Apple technologies

Integrate Game Center to help players discover your game across their devices and connect with their friends. Game Center is Apple's social gaming network, available on all platforms. Game Center lets players keep track of their progress and achievements and allows you to set up leaderboards, challenges, and multiplayer activities in your game. For design guidance, see Game Center; for developer guidance, see GameKit.

Let players pick up their game on any of their devices. People often have a single iCloud account that they use across multiple Apple devices. When you support GameSave, you can help people save their game state and start back up exactly where they left off on a different device.

Support haptics to help players feel the action. When you adopt Core Haptics, you can compose and play custom haptic patterns, optionally combined with custom audio content. Core Haptics is available in iOS, iPadOS, tvOS, and visionOS, and supported on many game controllers. For guidance, see Playing haptics; for developer guidance, see Core Haptics and Playing Haptics on Game Controllers.

Use Spatial Audio to immerse players in your game's soundscape. Providing multichannel audio can help your game's audio adapt automatically to the current device, enabling an immersive Spatial Audio experience where supported. For guidance, see Playing audio > visionOS; for developer guidance, see Explore Spatial Audio.

Take advantage of Apple technologies to enable unique gameplay mechanics. For example, you can integrate technologies like augmented reality, machine learning, and HealthKit, and request access to location data and functionality like camera and microphone. For a full list of Apple technologies, features, and services, see Technologies.

#### Resources

**Related**
- Game Center
- Game controls

**Developer documentation**
- Games Pathway
- Create games for Apple platforms

#### Change log

| Date | Changes |
|---|---|
| June 9, 2025 | Updated guidance for touch-based controls and Game Center. |
| June 10, 2024 | New page. |

---

### Designing for iOS
**Path:** Getting started  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/designing-for-ios

#### Hero image

![Designing for iOS](images/platforms-iOS-intro%402x.png)
*A stylized representation of an iPhone frame shown on top of a grid. The image is overlaid with rectangular and circular grid lines and is tinted green to subtly reflect the green in the original six-color Apple logo.*

#### Summary

People depend on their iPhone to help them stay connected, play games, view media, accomplish tasks, and track personal data in any location and while on the go.

As you begin designing your app or game for iOS, start by understanding the following fundamental device characteristics and patterns that distinguish the iOS experience. Using these characteristics and patterns to inform your design decisions can help you provide an app or game that iPhone users appreciate.

Display. iPhone has a medium-size, high-resolution display.

Ergonomics. People generally hold their iPhone in one or both hands as they interact with it, switching between landscape and portrait orientations as needed. While people are interacting with the device, their viewing distance tends to be no more than a foot or two.

Inputs. Multi-Touch gestures, virtual keyboards, and voice control let people perform actions and accomplish meaningful tasks while they're on the go. In addition, people often want apps to use their personal data and input from the device's gyroscope and accelerometer, and they may also want to participate in spatial interactions.

App interactions. Sometimes, people spend just a minute or two checking on event or social media updates, tracking data, or sending messages. At other times, people can spend an hour or more browsing the web, playing games, or enjoying media. People typically have multiple apps open at the same time, and they appreciate switching frequently among them.

System features. iOS provides several features that help people interact with the system and their apps in familiar, consistent ways.

- Widgets
- Home Screen quick actions
- Spotlight
- Shortcuts
- Activity views

#### Best practices

Great iPhone experiences integrate the platform and device capabilities that people value most. To help your design feel at home in iOS, prioritize the following ways to incorporate these features and capabilities.

- Help people concentrate on primary tasks and content by limiting the number of onscreen controls while making secondary details and actions discoverable with minimal interaction.
- Adapt seamlessly to appearance changes — like device orientation, Dark Mode, and Dynamic Type — letting people choose the configurations that work best for them.
- Support interactions that accommodate the way people usually hold their device. For example, it tends to be easier and more comfortable for people to reach a control when it's located in the middle or bottom area of the display, so it's especially important let people swipe to navigate back or initiate actions in a list row.
- With people's permission, integrate information available through platform capabilities in ways that enhance the experience without asking people to enter data. For example, you might accept payments, provide security through biometric authentication, or offer features that use the device's location.

#### Resources

**Related**
- Apple Design Resources

**Developer documentation**
- iOS Pathway

---

### Designing for iPadOS
**Path:** Getting started  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/designing-for-ipados

#### Hero image

![Designing for iPadOS](images/platforms-iPadOS-intro%402x.png)
*A stylized representation of an iPad frame shown on top of a grid. The image is overlaid with rectangular and circular grid lines and is tinted green to subtly reflect the green in the original six-color Apple logo.*

#### Summary

People value the power, mobility, and flexibility of iPad as they enjoy media, play games, perform detailed productivity tasks, and bring their creations to life.

As you begin designing your app or game for iPad, start by understanding the following fundamental device characteristics and patterns that distinguish the iPadOS experience. Using these characteristics and patterns to inform your design decisions can help you provide an app or game that iPad users appreciate.

Display. iPad has a large, high-resolution display.

Ergonomics. People often hold their iPad while using it, but they might also set it on a surface or place it on a stand. Positioning the device in different ways can change the viewing distance, although people are typically within about 3 feet of the device as they interact with it.

Inputs. People can interact with iPad using Multi-Touch gestures and virtual keyboards, an attached keyboard or pointing device, Apple Pencil, or voice, and they often combine multiple input modes.

App interactions. Sometimes, people perform a few quick actions on their iPad. At other times, they spend hours immersed in games, media, content creation, or productivity tasks. People frequently have multiple apps open at the same time, and they appreciate viewing more than one app onscreen at once and taking advantage of inter-app capabilities like drag and drop.

System features. iPadOS provides several features that help people interact with the system and their apps in familiar, consistent ways.

- Multitasking
- Widgets
- Drag and drop

#### Best practices

Great iPad experiences integrate the platform and device capabilities that people value most. To help your experience feel at home in iPadOS, prioritize the following ways to incorporate these features and capabilities.

- Take advantage of the large display to elevate the content people care about, minimizing modal interfaces and full-screen transitions, and positioning onscreen controls where they're easy to reach, but not in the way.
- Use viewing distance and input mode to help you determine the size and density of the onscreen content you display.
- Let people use Multi-Touch gestures, a physical keyboard or trackpad, or Apple Pencil, and consider supporting unique interactions that combine multiple input modes.
- Adapt seamlessly to appearance changes — like device orientation, multitasking modes, Dark Mode, and Dynamic Type — and transition effortlessly to running in macOS, letting people choose the configurations that work best for them.

#### Resources

**Related**
- Apple Design Resources

**Developer documentation**
- iPadOS Pathway

---

### Designing for macOS
**Path:** Getting started  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/designing-for-macos

#### Hero image

![Designing for macOS](images/platforms-macOS-intro%402x.png)
*A stylized representation of a Mac shown on top of a grid. The image is overlaid with rectangular and circular grid lines and is tinted green to subtly reflect the green in the original six-color Apple logo.*

#### Summary

People rely on the power, spaciousness, and flexibility of a Mac as they perform in-depth productivity tasks, view media or content, and play games, often using several apps at once.

As you begin designing your app or game for macOS, start by understanding the fundamental device characteristics and patterns that distinguish the macOS experience. Using these characteristics and patterns to inform your design decisions can help you provide an app or game that Mac users appreciate.

Display. A Mac typically has a large, high-resolution display, and people can extend their workspace by connecting additional displays, including their iPad.

Ergonomics. People generally use a Mac while they're stationary, often placing the device on a desk or table. In the typical use case, the viewing distance can range from about 1 to 3 feet.

Inputs. People expect to enter data and control the interface using any combination of input modes, such as physical Keyboards, Pointing devices, Game controls, and Siri.

App interactions. Interactions can last anywhere from a few minutes of performing some quick tasks to several hours of deep concentration. People frequently have multiple apps open at the same time, and they expect smooth transitions between active and inactive states as they switch from one app to another.

System features. macOS provides several features that help people interact with the system and their apps in familiar, consistent ways.

- The menu bar
- File management
- Going full screen
- Dock menus

#### Best practices

Great Mac experiences integrate the platform and device capabilities that people value most. To help your design feel at home in macOS, prioritize the following ways to incorporate these features and capabilities.

- Leverage large displays to present more content in fewer nested levels and with less need for modality, while maintaining a comfortable information density that doesn't make people strain to view the content they want.
- Let people resize, hide, show, and move your windows to fit their work style and device configuration, and support full-screen mode to offer a distraction-free context.
- Use the menu bar to give people easy access to all the commands they need to do things in your app.
- Help people take advantage of high-precision input modes to perform pixel-perfect selections and edits.
- Handle keyboard shortcuts to help people accelerate actions and use keyboard-only work styles.
- Support personalization, letting people customize toolbars, configure windows to display the views they use most, and choose the colors and fonts they want to see in the interface.

#### Resources

**Related**
- Apple Design Resources

**Developer documentation**
- macOS Pathway

---

### Designing for tvOS
**Path:** Getting started  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/designing-for-tvos

#### Hero image

![Designing for tvOS](images/platforms-tvOS-intro%402x.png)
*A stylized representation of a TV screen shown on top of a grid. The image is overlaid with rectangular and circular grid lines and is tinted green to subtly reflect the green in the original six-color Apple logo.*

#### Summary

People enjoy the vibrant content, immersive experiences, and streamlined interactions that tvOS delivers in media and games, as well as in fitness, education, and home utility apps.

As you begin designing your app or game for tvOS, start by understanding the following fundamental device characteristics and patterns that distinguish the tvOS experience. Using these characteristics and patterns to inform your design decisions can help you provide an app or game that tvOS users appreciate.

Display. A TV typically has a very large, high-resolution display.

Ergonomics. Although people generally remain many feet away from their stationary TV — often 8 feet or more — they sometimes continue to interact with content as they move around the room.

Inputs. People can use a remote, a game controller, their voice, and apps running on their other devices to interact with Apple TV.

App interactions. People can get deeply immersed in a single experience — often lasting hours — but they also appreciate using a picture-in-picture view to simultaneously follow an alternative app or video.

System features. Apple TV users expect their apps and games to integrate well with the following system experiences.

- Integrating with the TV app
- SharePlay
- Top Shelf
- TV provider accounts

#### Best practices

Great tvOS experiences integrate the platform and device capabilities that people value most. To help your experience feel at home in tvOS, prioritize the following ways to incorporate these features and capabilities.

- Support powerful, delightful interactions through the fluid, familiar gestures people make with the Siri Remote.
- Embrace the tvOS focus system, letting it gently highlight and expand onscreen items as people move among them, helping them know what to do and where they are at all times.
- Deliver beautiful, edge-to-edge artwork, subtle and fluid animations, and engaging audio, wrapping people in a rich, cinematic experience that's clear, legible, and captivating from across the room.
- Enhance multiuser support by making sign-in easy and infrequent, handling shared sign-in, and automatically switching profiles when people change the current viewer.

#### Resources

**Related**
- Apple Design Resources

**Developer documentation**
- tvOS Pathway

#### Change log

| Date | Changes |
|---|---|
| September 14, 2022 | Refined best practices for multiuser support. |

---

### Designing for visionOS
**Path:** Getting started  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/designing-for-visionos

#### Hero image

![Designing for visionOS](images/platforms-visionOS-intro%402x.png)
*A stylized representation of Apple Vision Pro shown on top of a grid. The image is overlaid with rectangular and circular grid lines and is tinted green to subtly reflect the green in the original six-color Apple logo.*

#### Summary

When people wear Apple Vision Pro, they enter an infinite 3D space where they can engage with your app or game while staying connected to their surroundings.

As you begin designing your app or game for visionOS, start by understanding the fundamental device characteristics and patterns that distinguish the platform. Use these characteristics and patterns to inform your design decisions and help you create immersive and engaging experiences.

Space. Apple Vision Pro offers a limitless canvas where people can view virtual content like windows, volumes, and 3D objects, and choose to enter deeply immersive experiences that can transport them to different places.

Immersion. In a visionOS app, people can fluidly transition between different levels of immersion. By default, an app launches in the Shared Space where multiple apps can run side-by-side and people can open, close, and relocate windows. People can also choose to transition an app to a Full Space, where it's the only app running. While in a Full Space app, people can view 3D content blended with their surroundings, open a portal to view another place, or enter a different world.

Passthrough. Passthrough provides live video from the device's external cameras, and helps people interact with virtual content while also seeing their actual surroundings. When people want to see more or less of their surroundings, they use the Digital Crown to control the amount of passthrough.

Spatial Audio. Apple Vision Pro combines acoustic and visual-sensing technologies to model the sonic characteristics of a person's surroundings, automatically making audio sound natural in their space. When an app receives a person's permission to access information about their surroundings, it can fine-tune Spatial Audio to bring custom experiences to life.

Eyes and hands. In general, people perform most actions by using their eyes to look at a virtual object and making an indirect gesture, like a tap, to activate it. People can also interact with a virtual object by using a direct gesture, like touching it with a finger.

Ergonomics. While wearing Apple Vision Pro, people rely entirely on the device's cameras for everything they see, both real and virtual, so maintaining visual comfort is paramount. The system helps maintain comfort by automatically placing content so it's relative to the wearer's head, regardless of the person's height or whether they're sitting, standing, or lying down. Because visionOS brings content to people — instead of making people move to reach the content — people can remain at rest while engaging with apps and games.

Accessibility. Apple Vision Pro supports accessibility technologies like VoiceOver, Switch Control, Dwell Control, Guided Access, Head Pointer, and many more, so people can use the interactions that work for them. In visionOS, as in all platforms, system-provided UI components build in accessibility support by default, while system frameworks give you ways to enhance the accessibility of your app or game.

> Important When building your app for Apple Vision Pro, be sure to consider the unique characteristics of the device and its spatial computing environment, and pay special attention to your user's safety; for more details about these characteristics, see Apple Vision Pro User Guide. For example, Apple Vision Pro should not be used while operating a vehicle or heavy machinery. The device is also not designed to be used while moving around unsafe environments such as near balconies, streets, stairs, or other potential hazards. Note that Apple Vision Pro is designed to be fit and used only by individuals 13 years of age or older.

#### Best practices

Great visionOS apps and games are approachable and familiar, while offering extraordinary experiences that can surround people with beautiful content, expanded capabilities, and captivating adventures.

Embrace the unique features of Apple Vision Pro. Take advantage of space, Spatial Audio, and immersion to bring life to your experiences, while integrating passthrough and spatial input from eyes and hands in ways that feel at home on the device.

Consider different types of immersion as you design ways to present your app's most distinctive moments. You can present experiences in a windowed, UI-centric context, a fully immersive context, or something in between. For each key moment in your app, find the minimum level of immersion that suits it best — don't assume that every moment needs to be fully immersive.

Use windows for contained, UI-centric experiences. To help people perform standard tasks, prefer standard windows that appear as planes in space and contain familiar controls. In visionOS, people can relocate windows anywhere they want, and the system's dynamic scaling helps keep window content legible whether it's near or far.

Prioritize comfort. To help people stay comfortable and physically relaxed as they interact with your app or game, keep the following fundamentals in mind.

- Display content within a person's field of view, positioning it relative to their head. Avoid placing content in places where people have to turn their head or change their position to interact with it.
- Avoid displaying motion that's overwhelming, jarring, too fast, or missing a stationary frame of reference.
- Support indirect gestures that let people interact with apps while their hands rest in their lap or at their sides.
- If you support direct gestures, make sure the interactive content isn't too far away and that people don't need to interact with it for extended periods.
- Avoid encouraging people to move too much while they're in a fully immersive experience.

Help people share activities with others. When you use SharePlay to support shared activities, people can view the spatial Personas of other participants, making it feel like everyone is together in the same space.

#### Resources

**Related**
- Apple Design Resources

**Developer documentation**
- visionOS Pathway
- Creating your first visionOS app

#### Change log

| Date | Changes |
|---|---|
| February 2, 2024 | Included a link to Apple Vision Pro User Guide. |
| September 12, 2023 | Updated intro artwork. |
| June 21, 2023 | New page. |

---

### Designing for watchOS
**Path:** Getting started  
**Canonical URL:** https://developer.apple.com/design/human-interface-guidelines/designing-for-watchos

#### Hero image

![Designing for watchOS](images/platforms-watchOS-intro%402x.png)
*A stylized representation of an Apple Watch frame shown on top of a grid. The image is overlaid with rectangular and circular grid lines and is tinted green to subtly reflect the green in the original six-color Apple logo.*

#### Summary

When people glance at their Apple Watch, they know they can access essential information and perform simple, timely tasks whether they're stationary or in motion.

As you begin designing your app for Apple Watch, start by understanding the following fundamental device characteristics and patterns that distinguish the watchOS experience. Using these characteristics and patterns to inform your design decisions can help you provide an app that Apple Watch users appreciate.

Display. The small Apple Watch display fits on the wrist while delivering an easy-to-read, high-resolution experience.

Ergonomics. Because people wear Apple Watch, they're usually no more than a foot away from the display as they raise their wrist to view it and use their opposite hand to interact with the device. In addition, the Always On display lets people view information on the watch face when they drop their wrist.

Inputs. People can navigate vertically or inspect data by turning the Digital Crown, which offers consistent control on the watch face, the Home Screen, and within apps. They can provide input even while they're in motion with standard gestures like tap, swipe, and drag. Pressing the Action button initiates an essential action without looking at the screen, and using shortcuts helps people perform their routine tasks quickly and easily. People can also take advantage of data that device features provide, such as GPS, sensors for blood oxygen and heart function, and the altimeter, accelerometer, and gyroscope.

App interactions. People glance at the Always On display many times throughout the day, performing concise app interactions that can last for less than a minute each. People frequently use a watchOS app's related experiences — like complications, notifications, and Siri interactions — more than they use the app itself.

System features. watchOS provides several features that help people interact with the system and their apps in familiar, consistent ways.

- Complications
- Notifications
- Always On
- Watch faces

#### Best practices

Great Apple Watch experiences are streamlined and specialized, and integrate the platform and device capabilities that people value most. To help your experience feel at home in watchOS, prioritize the following ways to incorporate these features and capabilities.

- Support quick, glanceable, single-screen interactions that deliver critical information succinctly and help people perform targeted actions with a simple gesture or two.
- Minimize the depth of hierarchy in your app's navigation, and use the Digital Crown to provide vertical navigation for scrolling or switching between screens.
- Personalize the experience by proactively anticipating people's needs and using on-device data to provide actionable content that's relevant in the moment or very soon.
- Use complications to provide relevant, potentially dynamic data and graphics right on the watch face where people can view them on every wrist raise and tap them to dive straight into your app.
- Use notifications to deliver timely, high-value information and let people perform important actions without opening your app.
- Use background content such as color to convey useful supporting information, and use materials to illustrate hierarchy and a sense of place.
- Design your app to function independently, complementing your notifications and complications by providing additional details and functionality.

#### Resources

**Related**
- Apple Design Resources

**Developer documentation**
- watchOS Pathway

#### Change log

| Date | Changes |
|---|---|
| June 5, 2023 | Enhanced guidance for providing a glanceable, focused app experience, and emphasized the importance of the Digital Crown in navigation. |

---
