## Inputs

the Human Interface Guidelines define how apps receive input from users across all Apple platforms. Input mechanisms range from physical hardware (Action Button, Digital Crown, keyboards, remotes) to touch and gesture surfaces (multi-touch, Apple Pencil, Camera Control) to spatial and sensor-based input (eyes, gyroscope, accelerometer, nearby interactions). This reference covers all 13 HIG Inputs pages in full detail, distilled from the live HIG for macOS 26 Tahoe and aligned platforms.

| Page | Slug | Coverage |
| --- | --- | --- |
| Action button | action-button | Detailed |
| Apple Pencil and Scribble | apple-pencil-and-scribble | Detailed |
| Camera control | camera-control | Detailed |
| Digital Crown | digital-crown | Detailed |
| Eyes | eyes | Detailed |
| Focus and selection | focus-and-selection | Detailed |
| Game controls | game-controls | Detailed |
| Gestures | gestures | Detailed |
| Gyroscope and accelerometer | gyro-and-accelerometer | Detailed |
| Keyboards | keyboards | Detailed |
| Nearby interactions | nearby-interactions | Detailed |
| Pointing devices | pointing-devices | Detailed |
| Remotes | remotes | Detailed |

---

## Action button

![Action button hero](https://docs-assets.developer.apple.com/published/e9f0fbe50cec4b4b3ac0ef5daa14e321/inputs-action-button-intro@2x.png)

Source: https://developer.apple.com/design/human-interface-guidelines/action-button

The Action button is a customizable hardware button available on iPhone 15 Pro and iPhone 15 Pro Max and later, and on Apple Watch Ultra and Apple Watch Ultra 2. By default, the button silences or rings the device, but people can assign a different built-in action or a Shortcut to it. Apps can provide custom actions for the Action button in their Info.plist so people can make app-specific tasks available at a press.

Apps can also register for Action button input, letting people use the Action button as an analog input control within supported app experiences — for example, as a shutter button in a camera app or a push-to-talk button in a walkie-talkie experience.

When an app receives an Action button press, the system activates the app if it isn't already frontmost before delivering the event.

#### Best practices

- **Provide a meaningful, focused action.** The Action button lets people quickly initiate a single high-value task. Offer an action that's immediately useful, has a clear relationship to your app, and can be described briefly in a few words.
- **Prefer actions that produce an immediate result.** Avoid actions that require additional input before doing anything. If additional input is unavoidable, provide it as a Shortcut rather than a built-in action so people know what to expect.
- **Avoid replicating system-level behaviors.** The system already offers built-in actions like Mute, Flashlight, Focus, and Camera. Don't offer custom actions that duplicate these unless your implementation provides meaningfully enhanced behavior.

#### Platform considerations

**iOS**

The Action button is available on iPhone 15 Pro, iPhone 15 Pro Max, and later. Register your app's custom actions in your app's Info.plist. Each action entry provides a title, an optional subtitle, a system image name, and the intent or Shortcut to perform.

**watchOS**

On Apple Watch Ultra and Apple Watch Ultra 2, the Action button is a physical button separate from the Digital Crown and side button. Apps can register custom Workout, Dive, and custom actions. The system presents a list of registered actions when the user long-presses the Action button from the watch face.

#### Resources

- Human Interface Guidelines: Action button — https://developer.apple.com/design/human-interface-guidelines/action-button
- Developer documentation: UIApplicationShortcutItem, WKExtendedRuntimeSession

#### Change log

| Date | Change |
| --- | --- |
| Sep 12, 2023 | Updated to include guidance for iOS. |
| Sep 14, 2022 | New page. |

---

## Apple Pencil and Scribble

![Apple Pencil and Scribble hero](https://docs-assets.developer.apple.com/published/f6d1b0f1a9c2f4e3c8e7d5b2a0e9f3c1/inputs-apple-pencil-and-scribble-intro@2x.png)

Source: https://developer.apple.com/design/human-interface-guidelines/apple-pencil-and-scribble

Apple Pencil is a precision stylus that lets people write, draw, annotate, and navigate content on iPad with high accuracy and low latency. Different Apple Pencil models support different interaction methods, and apps can enhance their experience by handling all supported input types. Scribble allows people to write with Apple Pencil directly into any text field, and the system automatically converts the handwriting to typed text.

#### Best practices

- **Support all Apple Pencil interactions your app can benefit from.** Even if writing and drawing are your primary use cases, consider supporting hover, double tap, squeeze, and barrel roll to give people more ways to interact with your content.
- **Give immediate visual feedback for Pencil interactions.** When someone taps, double-taps, or squeezes, respond instantly so the interaction feels direct and physical.
- **Never use Apple Pencil interactions as the only way to perform a critical action.** Offer equivalent ways to perform the same actions with touch or a pointer so all users can access your app's functionality.
- **Make your custom drawing experience feel natural.** Match the rendering speed and accuracy of the system; avoid using GPU-intensive post-processing that slows rendering below the display's refresh rate.

#### Hover

Apple Pencil Pro and Apple Pencil (2nd generation) support hover detection, which lets your app respond to the Pencil being near the screen before it makes contact. The system reports hover distance, altitude angle, and azimuth angle.

- Use hover to show a preview of what the Pencil will do when it lands — for example, show a paint stroke preview, a cursor, or a tooltip.
- Avoid making hover interactions disruptive or distracting if the user isn't actively looking at the hover target.
- Don't require hover as the only trigger for revealing controls; people using earlier Apple Pencil models won't have hover available.

#### Double tap

Double tap on the flat side of Apple Pencil (2nd generation) or Apple Pencil Pro lets people quickly switch between tools in your app without lifting the Pencil from the screen or reaching for the toolbar.

- Provide a meaningful two-state toggle for double tap — for example, switch between the current drawing tool and the eraser, or toggle between a tool and the selection mode.
- Show clear visual feedback when the tool switches so the user knows what mode they're in.
- Don't use double tap to trigger destructive actions like deleting content.

#### Squeeze

Apple Pencil Pro supports a squeeze gesture that people perform by squeezing the barrel of the Pencil. This gesture can trigger actions that benefit from a quick, one-handed interaction.

> Note: Squeeze is available only on Apple Pencil Pro. Design your squeeze interaction to be a convenience, not a requirement.

- Use squeeze to open a contextual palette or a tool picker near the Pencil tip.
- Keep the palette small and dismissible; it should be easy to quickly choose an option and return to drawing.
- Avoid using squeeze for actions that are difficult to reverse.

#### Barrel roll

Apple Pencil Pro reports the barrel roll angle — the rotation of the Pencil around its own long axis. This input is useful in creative apps where rotation of a real brush or marker would affect the stroke.

- Use barrel roll in artistic or design apps to affect stroke shape, brush texture, or tool angle.
- Provide a visual indicator of the current barrel roll angle if it meaningfully affects the output.
- Don't require barrel roll for primary functionality; use it as an enhancement for people who want expressive control.

#### Scribble

Scribble converts handwriting into typed text in any text field. Apps that use standard UITextField and UITextView controls get Scribble support automatically. Custom text views can adopt the UIIndirectScribbleInteraction and UIScribbleInteraction APIs to support Scribble.

- Adopt Scribble in custom text input controls so people can write naturally in your app.
- Don't disable Scribble in standard text fields unless your app has a specific handwriting recognition or ink-preservation workflow that would be disrupted.
- Consider supporting Scribble gestures (scratch-out to delete, circle to select) in custom drawing views by implementing the appropriate delegate callbacks.

#### Custom drawing

Apps that handle raw Apple Pencil input — such as illustration, note-taking, or signature-capture apps — should use the low-latency drawing APIs to provide the most responsive experience.

- Use PencilKit or the low-latency event delivery APIs to minimize stroke latency.
- Support predicted touch events to reduce perceptible lag at high display refresh rates.
- Render ink incrementally rather than re-drawing the entire canvas on every event.
- Store strokes in a model that can be re-rendered at different scales and resolutions.

#### Platform considerations

Apple Pencil is supported on iPad only. No additional guidance for iPhone, Mac, Apple TV, Apple Watch, or Apple Vision Pro beyond what is covered above.

#### Resources

**Related**
- Apple Pencil (product page) — https://www.apple.com/apple-pencil/
- PencilKit — https://developer.apple.com/documentation/pencilkit

**Developer documentation**
- UIIndirectScribbleInteraction
- UIScribbleInteraction
- Apple Pencil interactions

**Videos**
- Meet PencilKit — WWDC19
- Pencil Interactions in iPad Apps — WWDC19
- What's new in PencilKit — WWDC20

#### Change log

| Date | Change |
| --- | --- |
| May 7, 2024 | Added guidance for Apple Pencil Pro squeeze and barrel roll. |
| Sep 12, 2023 | Updated artwork. |
| Nov 3, 2022 | Added guidance for hover. |

---

## Camera control

![Camera control hero](https://docs-assets.developer.apple.com/published/c2a1b3d4e5f6a7b8c9d0e1f2a3b4c5d6/inputs-camera-control-intro@2x.png)

Source: https://developer.apple.com/design/human-interface-guidelines/camera-control

Camera Control is a hardware button on iPhone 16 and later that provides direct, tactile access to camera functions. It supports pressing, clicking, swiping, and two-stage press interactions. By default it opens Camera and controls shutter, video recording, zoom, and exposure. Apps can use Camera Control to offer streamlined capture workflows.

#### Anatomy

Camera Control supports two types of interactions:

**Slider interactions** — people swipe along the button surface to adjust a continuous value:
- Zoom
- Exposure

**Picker interactions** — people swipe to cycle through discrete options:
- Camera mode
- Lens selection

System controls available through Camera Control:
- Zoom
- Exposure compensation
- Lens selection (on models with multiple cameras)

#### Best practices

- **Integrate Camera Control into your capture workflow when relevant.** If your app performs photo or video capture, adopt Camera Control APIs so people can use the physical button they already know.
- **Map Camera Control input to the most logical capture action.** In a camera app, the primary click should trigger capture; a two-stage press should half-press to focus and fully press to capture, matching the behavior of a traditional camera shutter.
- **Support the full interaction vocabulary.** Implement click, light press, full press, and swipe so people can use Camera Control exactly as they do in the system Camera app.
- **Provide clear visual feedback for Camera Control interactions.** Show focus confirmation, capture confirmation, and mode changes so the user understands what happened.
- **Don't repurpose Camera Control for non-capture tasks.** People associate Camera Control with photography and video; using it for unrelated actions creates confusion.
- **Handle the case where Camera Control is not available.** Camera Control is available on iPhone 16 and later; always provide alternative on-screen controls for older devices.
- **Respond to Camera Control input even when the screen is off (where appropriate).** On iPhone 16, pressing Camera Control while the screen is off can launch the camera. If your app registers as a camera, handle this launch path.
- **Support the two-stage press for focus and capture.** Stage one (light press) should prepare the capture — lock focus, lock exposure — and stage two (full press) should trigger it.
- **Keep Camera Control interactions fast and low-latency.** Hardware button interactions carry an expectation of immediacy; avoid heavy computation on the capture path.

#### Platform considerations

Camera Control is available on iPhone 16 and later only. No additional guidance for iPad, Mac, Apple TV, Apple Watch, or Apple Vision Pro.

#### Resources

- Human Interface Guidelines: Camera control — https://developer.apple.com/design/human-interface-guidelines/camera-control
- AVCaptureEventInteraction — https://developer.apple.com/documentation/avkit/avcaptureeventinteraction

#### Change log

| Date | Change |
| --- | --- |
| Sep 9, 2024 | New page. |

---

## Digital Crown

![Digital Crown hero](https://docs-assets.developer.apple.com/published/d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6/inputs-digital-crown-intro@2x.png)

Source: https://developer.apple.com/design/human-interface-guidelines/digital-crown

The Digital Crown is a rotating input device found on Apple Watch and Apple Vision Pro. On Apple Watch it serves as the primary navigation control; on Apple Vision Pro it is the primary brightness and volume control. Apps can use Digital Crown input for scrolling, adjusting values, and custom interactions.

#### Apple Vision Pro

On Apple Vision Pro, the Digital Crown supports the following system-level interactions that apps should not override:

- Brightness adjustment (rotate)
- Volume adjustment (rotate while audio is playing)
- Return to Home (press)
- Siri (press and hold)
- Mute (double press during a call)
- Take a screenshot (press and hold + top button)

#### Apple Watch

The Digital Crown on Apple Watch is the primary scrolling and navigation control. Apps should support Digital Crown input for any scrollable content.

- **Use the Digital Crown for scrolling wherever appropriate.** All scrollable lists and content views should respond to Digital Crown rotation so people can scroll without touching the screen.
- **Provide focused, single-axis Crown interaction in your main view.** Assign one clear behavior — scroll, zoom, or value adjustment — to the Crown for each screen to avoid confusion.
- **Give real-time visual feedback as the Crown turns.** Animate content in direct proportion to Crown input; don't apply snapping or lag unless it's intentional and immediately perceptible.
- **Use the Crown for value pickers.** Let people adjust time pickers, sliders, and segmented controls by rotating the Crown, so they don't have to precisely tap small controls.
- **Pair Crown rotation with haptic feedback.** Use WKHapticType.click haptics to create a tactile detent sensation as values increment or list items are traversed.
- **Consider using the Digital Crown for custom experiences.** For games, measurement tools, or creative apps, the Crown can serve as an analog dial or jog wheel.
- **Handle Crown presses appropriately.** A Crown press returns the user to the watch face or the previous screen; don't intercept it unless you have a very specific reason.
- **Support both rotation directions.** Clockwise and counter-clockwise Crown rotation should always produce symmetric, logical results.
- **Don't assign too many modes to the Crown.** If the meaning of Crown rotation changes contextually, make the current mode extremely obvious through the UI.
- **Test Crown interaction with the physical device.** Crown feel and the relationship between rotation angle and value change differs between simulator and hardware.

> Note: On Apple Watch Ultra and Apple Watch Ultra 2, the Action button provides a second hardware input that is separate from the Digital Crown. Assign your highest-priority custom action to the Action button and use the Digital Crown for continuous control.

#### Platform considerations

No additional guidance for iOS, iPadOS, macOS, or tvOS. The Digital Crown is specific to Apple Watch and Apple Vision Pro.

#### Resources

- Human Interface Guidelines: Digital Crown — https://developer.apple.com/design/human-interface-guidelines/digital-crown
- WKCrownDelegate — https://developer.apple.com/documentation/watchkit/wkcrowndelegate

#### Change log

| Date | Change |
| --- | --- |
| Dec 5, 2023 | Updated guidance for Apple Vision Pro. |
| Jun 21, 2023 | Updated for watchOS 10. |
| Jun 5, 2023 | Added guidance for Apple Vision Pro. |

---

## Eyes

![Eyes hero](https://docs-assets.developer.apple.com/published/a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6/inputs-eyes-intro@2x.png)

Source: https://developer.apple.com/design/human-interface-guidelines/eyes

> Important: Eye tracking data is extremely sensitive personal information. ARKit infers where people are looking but never exposes raw gaze data to apps. Apps must request permission through standard privacy APIs before using eye-tracking features, and must clearly explain in their privacy policy and permission request how the data is used.

On Apple Vision Pro, eye tracking is the primary pointing mechanism. People look at interface elements to focus them, then use pinch gestures or voice commands to activate them. Understanding how eye-based focus works helps you design interfaces that feel natural and effortless to use.

#### Best practices

- **Design for eye-based interaction from the start.** Eye tracking as a primary input modality changes how focus, selection, and confirmation should be architected; don't just port a touch-based interface.
- **Never display or log where the user is looking.** The system restricts access to raw gaze data; design your features around the intent-based hit-testing APIs, not raw eye coordinates.
- **Provide alternatives to eye-based interaction.** Support pointer input from a trackpad or Magic Keyboard so people who find eye-based interaction difficult can still use your app.
- **Make it easy to activate the intended target.** If people frequently activate neighboring items accidentally, your layout needs larger targets or more spacing.

#### Making items easy to see

- **Size interactive elements generously.** Targets should be at least 60x60 pt. Smaller elements are harder to focus and easier to overshoot.
- **Use high contrast between interactive elements and their backgrounds.** Elements need to be visually distinct at a glance because the user is looking, not reaching.
- **Provide sufficient spacing between interactive elements.** Items that are too close together cause false activations and frustration.

#### Encouraging interaction

- **Give clear visual affordances.** Make it obvious what can be interacted with so users know where to look.
- **Use hover effects to confirm focus.** The standard system hover highlight shows people exactly which element they're looking at; don't suppress it unless you're providing a clearly superior alternative.
- **Keep the interaction model simple.** Avoid requiring people to look at a specific element for an extended period or to perform precise saccades between small targets.

#### Custom hover effects

You can customize the appearance of the system hover highlight that appears when someone looks at an interactive element. Custom hover effects should enhance clarity rather than impede it.

- **Use hover effects to reveal secondary information.** A hover can show a label, badge, or tooltip that adds context without cluttering the resting state.
- **Keep hover transitions quick.** A hover effect that fades in slowly creates a noticeable lag between where the user is looking and when the UI responds.
- **Avoid hover effects that move content.** Shifting layouts or repositioning elements while someone is looking at them can feel unstable and disorienting.
- **Don't use hover to reveal content that is required for interaction.** Primary labels, prices, or actions that people need before deciding to activate an element should be visible without hover.
- **Avoid high-contrast or strongly saturated hover colors that shift perceived brightness.** Subtle scale, brightness, or saturation changes are more comfortable than abrupt color flips.
- **Prefer system-provided hover styles when possible.** The system hover effect is tuned to be comfortable for extended eye-tracking sessions; custom alternatives should meet the same bar.
- **Offer appropriate hover delay types:**
  - `.automatic` — system-chosen delay based on context
  - `.none` — hover activates instantly on look
  - `.after(TimeInterval)` — hover activates after a custom delay
- **Test hover effects across different lighting conditions** in a real device environment.
- **Don't make hover the only visual state change.** Combine hover with accessible alternatives like accessible color schemes and Dynamic Type.

#### Platform considerations

Eyes input is specific to Apple Vision Pro. No additional guidance for iOS, iPadOS, macOS, tvOS, or watchOS.

#### Resources

- Human Interface Guidelines: Eyes — https://developer.apple.com/design/human-interface-guidelines/eyes
- ARKit eye tracking — https://developer.apple.com/documentation/arkit

#### Change log

| Date | Change |
| --- | --- |
| Jun 10, 2024 | Updated guidance for visionOS 2. |
| Mar 29, 2024 | Added custom hover effects guidance. |
| Oct 24, 2023 | Added platform considerations detail. |
| Jun 21, 2023 | New page. |

---

## Focus and selection

![Focus and selection hero](https://docs-assets.developer.apple.com/published/b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6/inputs-focus-and-selection-intro@2x.png)

Source: https://developer.apple.com/design/human-interface-guidelines/focus-and-selection

Focus is the mechanism the system uses to indicate which element is ready to receive input when a pointing device, remote, keyboard, or eye-based input is used. Selection reflects the element the user has activated or chosen. Understanding focus and selection is essential for building accessible, keyboard- and remote-navigable interfaces.

#### Best practices

- **Ensure every interactive element can receive focus.** If an element can be tapped or clicked, it must be reachable by keyboard Tab, arrow keys, or remote D-pad so all users can operate it.
- **Make the current focus state visually clear.** The system provides default focus appearances; use them or provide a clearly visible custom alternative. Never remove focus indicators entirely.
- **Maintain logical focus order.** Focus should move through the interface in a reading-order sequence (top-to-bottom, leading-to-trailing) unless there is a compelling spatial reason to deviate.
- **Return focus to a sensible location after modal dismissal.** When a sheet, alert, or popover closes, restore focus to the control that opened it or the next logical element.
- **Trap focus inside modal contexts.** When a modal dialog or alert is visible, focus should cycle within it and not escape to background content.

#### Platform considerations

No additional guidance for iOS or watchOS.

**iPadOS**

On iPad with an external keyboard or pointer device, focus appears as a halo highlight around the focused element. Ensure your custom controls implement UIFocusEnvironment and respond to the `didUpdateFocus(in:with:)` callback.

Focus on iPad uses highlight focus (a semi-transparent rounded highlight) rather than the TV-style scale focus.

**tvOS**

tvOS uses a five-state focus system: unfocused, focused, pressed, highlighted, and disabled. Ensure custom focusable views handle all five states.

| State | Description |
| --- | --- |
| Unfocused | Default resting state |
| Focused | Element has focus; typically shown with scale and shadow lift |
| Pressed | User is pressing Select while focused |
| Highlighted | Used for certain context menu or menu interactions |
| Disabled | Element is present but not interactive |

**visionOS**

> Note: On visionOS, focus is driven by eye tracking. The system focus ring automatically highlights the element the user is looking at. Avoid creating custom focus effects that conflict with or suppress the system eye-tracking highlight.

#### Resources

- Human Interface Guidelines: Focus and selection — https://developer.apple.com/design/human-interface-guidelines/focus-and-selection
- UIFocusEnvironment — https://developer.apple.com/documentation/uikit/uifocusenvironment

#### Change log

| Date | Change |
| --- | --- |
| Oct 24, 2023 | Updated visionOS guidance. |
| Jun 21, 2023 | Updated for tvOS 17 and visionOS. |

---

## Game controls

![Game controls hero](https://docs-assets.developer.apple.com/published/c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6/inputs-game-controls-intro@2x.png)

Source: https://developer.apple.com/design/human-interface-guidelines/game-controls

Games on Apple platforms can receive input from three broad categories of control surface: touch controls on screen, physical game controllers connected over Bluetooth or USB, and keyboards. Supporting all three categories ensures your game is accessible to the widest audience on each platform.

People expect:
- Touch controls when using iPhone or iPad without an external controller
- Physical controller support on any platform that supports Bluetooth controllers

#### Touch controls

- **Design touch controls for the screen size.** On iPhone, controls must be reachable with thumbs while holding the device; on iPad, controls can be positioned to take advantage of the larger screen.
- **Make touch targets large enough to be reliably hit.** Game controls should be at minimum 44x44 pt; thumb-operated controls benefit from being 60-88 pt.
- **Use semi-transparent overlaid controls rather than opaque ones.** On-screen controls should not obstruct the game view more than necessary.
- **Allow players to customize control layout.** Let people move, resize, or hide on-screen controls to match their play style.
- **Show on-screen controls by default when no physical controller is connected.** Detect controller connection and disconnection and update the control visibility accordingly.
- **Fade out idle controls.** If on-screen controls haven't been used for several seconds, fade them to reduce visual clutter; restore them immediately on the next touch.
- **Support landscape and portrait where both make sense.** If your game can be played in both orientations, adapt control layout for each.
- **Avoid placing controls near the Home indicator and Dynamic Island.** Respect system safe areas so controls are clearly distinguishable from system gestures.
- **Provide haptic feedback for control interactions.** Use UIFeedbackGenerator or CoreHaptics to give tactile confirmation for button presses and significant game events.

#### Physical controllers

When a physical controller is connected, map your game actions to standard controller buttons. The following mapping is recommended:

| Game action | Controller input |
| --- | --- |
| Primary action / confirm | A (Cross on PlayStation) |
| Secondary action / cancel | B (Circle on PlayStation) |
| Tertiary action | X (Square on PlayStation) |
| Quaternary action | Y (Triangle on PlayStation) |
| Pause / menu | Menu button |
| Options / back | Options button |
| Sprint / modifier | Right trigger (R2) |
| Aim / alternate action | Left trigger (L2) |
| Camera control | Right thumbstick |
| Move | Left thumbstick |
| Quick action 1 | Right bumper (R1) |
| Quick action 2 | Left bumper (L1) |

#### Keyboards

- **Map standard keyboard keys to expected game actions.** WASD for movement, Space for jump or confirm, Escape for pause or back, and Enter for confirm are industry-standard expectations.
- **Show keyboard shortcut hints in your UI.** If a player is using a keyboard, display key labels next to actions in menus and HUD elements.
- **Support key remapping.** Let players assign their preferred keys to all bindable actions.
- **Handle keyboard focus correctly.** When a keyboard-navigable menu is open, ensure focus moves predictably with arrow keys and is confirmed with Enter.

#### Platform considerations

No additional guidance for iOS, iPadOS, macOS, or tvOS beyond what is described above.

Game controls are not applicable to watchOS.

**visionOS**

On visionOS, physical controllers are the recommended input method for games. Touch controls are not available. Design your control scheme to work entirely with physical controllers. You may also support keyboard input for menu navigation.

#### Resources

- Human Interface Guidelines: Game controls — https://developer.apple.com/design/human-interface-guidelines/game-controls
- Game Controller framework — https://developer.apple.com/documentation/gamecontroller

#### Change log

| Date | Change |
| --- | --- |
| Jun 9, 2025 | Updated for latest controller support. |
| Jun 10, 2024 | Added visionOS guidance. |

---

## Gestures

![Gestures hero](https://docs-assets.developer.apple.com/published/d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6/inputs-gestures-intro@2x.png)

Source: https://developer.apple.com/design/human-interface-guidelines/gestures

Gestures let people interact with content in a direct, physical way. Each Apple platform defines a vocabulary of standard gestures that people use every day; apps should use these gestures consistently so people can apply knowledge from the rest of the system to your app. Custom gestures are appropriate for specialized apps — especially games and creative tools — but require careful design to be discoverable.

#### Best practices

- **Use standard gestures for standard behaviors.** Swipe to go back, pinch to zoom, and two-finger scroll are deeply ingrained. Using them for other purposes will confuse people.
- **Don't remove or intercept system gestures.** System gestures like the home indicator swipe-up and edge-from-side swipes take priority; your app should not attempt to override them.
- **Provide multiple ways to access important features.** Don't rely on a single gesture as the only access point for a feature; offer a button or menu alternative.
- **Make gestures discoverable.** Gestures that aren't signaled by visible affordances require explicit teaching; use coach marks, empty-state hints, or onboarding flows to introduce non-obvious gestures.

#### Custom gestures

Custom gestures expand the interaction vocabulary of your app but carry a discoverability cost because they're unknown to new users.

- **Define custom gestures that feel physically motivated.** A twist to rotate, a spread to expand, and a push to reveal feel natural because they match the physical intuition of the action.
- **Avoid gesture conflicts with the system.** Test all custom gestures against the full set of system gesture recognizers to confirm there is no ambiguity.
- **Document custom gestures within the app.** Show a gesture hint in the UI, provide a gesture guide in onboarding, and list gestures in a Help menu.

#### Platform considerations

**iOS and iPadOS**

| Standard gesture | Standard behavior |
| --- | --- |
| Tap | Select or activate |
| Double tap | Zoom in; secondary action |
| Long press | Reveal context menu or begin drag |
| Swipe | Scroll; navigate back (leading edge) |
| Pinch | Zoom |
| Rotate | Rotate content |
| Two-finger tap | Undo (in some contexts) |
| Three-finger pinch | Copy (in some contexts) |

On iPadOS, gestures can be augmented with modifier keys from an attached keyboard.

**macOS**

On Mac, the primary gesture vocabulary is delivered through the trackpad. Standard trackpad gestures include two-finger scroll, pinch-to-zoom, two-finger swipe for page navigation, and force click. Apps that display zoomable content should respond to pinch and spread.

**tvOS**

On tvOS, the Siri Remote touch surface supports swipe, click, and press gestures. Apps should use the directional swipe for focus movement and the center click for selection. Custom swipe gestures on the remote touch surface are possible but should be used sparingly.

**visionOS**

visionOS supports indirect and direct gestures.

Indirect gestures (at a distance, no contact with a surface):
- Look to focus + pinch to select
- Pinch and drag to scroll or move
- Two-hand pinch to zoom
- Wrist flick to dismiss

Direct gestures (touching the virtual surface or a real surface):

| Direct gesture | Behavior |
| --- | --- |
| Tap | Activate |
| Double tap | Zoom or secondary action |
| Swipe | Scroll |
| Pinch | Zoom |
| Long press | Context menu |

- **Design for indirect gestures first.** Most users interact from a comfortable viewing distance without touching anything; your app's primary gestures should work at a distance.
- **Use direct gestures for immersive or creative experiences.** When proximity to content is part of the experience, direct touch gestures add value.
- **Don't require fine motor precision.** Indirect pinch at a distance has inherent variability; design targets and gesture thresholds to be forgiving.
- **Support the system dismissal gestures.** Allow the wrist flick and back-to-home Crown press to work without interference.

**Designing custom gestures in visionOS**

- Keep custom gestures simple — two hands performing simultaneous distinct actions is the limit for reliable detection.
- Provide visual previews showing how to perform the gesture before asking the user to execute it.
- Always provide a non-gesture alternative for every custom gesture action.
- Test custom gestures with users who have varying hand sizes and mobility.

**Working with system overlays in visionOS**

- System overlays (notification banners, volume HUD, Siri) can occlude your app; don't place critical controls in areas where overlays commonly appear.
- Your app should remain usable while overlays are visible; don't pause game or media playback just because an overlay is present.
- Resume any temporarily interrupted interaction after an overlay dismisses.
- Handle the case where a person has dismissed an overlay that was obscuring your UI — they may need to re-initiate a gesture.
- Use the UIScene observation APIs to detect when your scene is partially obscured and adapt accordingly.

**watchOS — Double tap**

On Apple Watch Series 9, Apple Watch Ultra 2, and later, people can perform a double tap gesture (bringing the index finger and thumb together twice) to activate the primary action of the current screen without touching the display.

- Assign the most important action on each screen to the double tap target.
- Ensure the double tap target is clearly identifiable as the primary action.
- Test double tap interaction across all screens in your app.

#### Specifications

**Standard gestures**

| Gesture | Minimum finger contact | Notes |
| --- | --- | --- |
| Tap | 1 | Activates items, confirms selections |
| Double tap | 1 | Zoom in; app-defined secondary action |
| Long press | 1 | 0.5 s hold; reveals context menu or begins drag |
| Swipe | 1 | Direction determines action; can be edge-triggered |
| Pinch | 2 | Open to zoom in, close to zoom out |
| Rotate | 2 | Rotates content or control |
| Three-finger swipe | 3 | System-reserved for undo/redo on iOS |

> Note: Three-finger gestures are partially reserved by the system for accessibility features. Test your three-finger gesture handling with AssistiveTouch enabled.

#### Resources

- Human Interface Guidelines: Gestures — https://developer.apple.com/design/human-interface-guidelines/gestures

#### Change log

| Date | Change |
| --- | --- |
| Sep 9, 2024 | Updated visionOS guidance for spatial computing. |
| Sep 15, 2023 | Added watchOS double tap guidance. |
| Jun 21, 2023 | Updated for visionOS. |

---

## Gyroscope and accelerometer

![Gyroscope and accelerometer hero](https://docs-assets.developer.apple.com/published/e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6/inputs-gyroscope-intro@2x.png)

Source: https://developer.apple.com/design/human-interface-guidelines/gyro-and-accelerometer

The gyroscope and accelerometer give apps access to real-time device motion data. Apps can use this data to create motion-driven interactions, augment reality experiences, detect gestures, and track physical activity. Both sensors are high-precision hardware components available on iPhone, iPad, and Apple Watch.

The accelerometer measures linear acceleration along three axes; the gyroscope measures rotational rate around three axes. Together they enable full six-degrees-of-freedom motion tracking. The Core Motion framework provides fused attitude data that combines both sensors to produce stable, drift-corrected device orientation.

#### Best practices

> Important: Motion data can infer sensitive information about people's activities, location, and health. Always request motion permission only when your app has an active, obvious need for it, and explain the purpose clearly in the NSMotionUsageDescription string in your Info.plist.

- **Use motion data for immersive or physical interaction use cases.** Tilt-to-steer in a game, step counting in a fitness app, and horizon leveling in a camera app are intuitive and appropriate; using motion to trigger unrelated UI changes is confusing.
- **Provide non-motion alternatives for motion-driven features.** Some people hold or use their devices in ways that don't produce the expected motion, and some accessibility needs preclude motion interaction. Always offer an equivalent touch-based control.

#### Platform considerations

No additional guidance for any specific platform beyond what is described above. Motion sensors are available on iPhone, iPad, Apple Watch, and supported iPod touch models.

#### Resources

**Related**
- Core Motion — https://developer.apple.com/documentation/coremotion

**Developer documentation**
- CMMotionManager
- CMAttitude
- CMDeviceMotion

**Videos**
- Core Motion sessions at WWDC

---

## Keyboards

![Keyboards hero](https://docs-assets.developer.apple.com/published/f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6/inputs-keyboard-intro@2x.png)

Source: https://developer.apple.com/design/human-interface-guidelines/keyboards

Keyboard input is a primary interaction modality on Mac and is increasingly important on iPad, iPhone (via external keyboards), and Apple Vision Pro. Supporting standard keyboard shortcuts reduces friction for power users and is essential for accessibility — users who rely on keyboard navigation cannot use apps that don't implement it.

The system provides standard keyboard shortcuts for common actions. Apps should implement all applicable shortcuts and may add custom shortcuts for app-specific features.

#### Best practices

> Important: Never use keyboard shortcuts as the only way to perform an action. Every keyboard-driven action must also be reachable by pointer, touch, or another input method.

- **Implement all relevant system-standard shortcuts.** People who use keyboards extensively develop muscle memory for standard shortcuts; surprising them with non-standard behavior erodes trust.
- **Provide discoverable keyboard shortcuts.** Show shortcuts in menus alongside the menu item label. On iPadOS, implement UIKeyCommand with discoverabilityTitle so shortcuts appear in the keyboard shortcut overlay.

#### Standard keyboard shortcuts

**System-wide shortcuts**

| Shortcut | Action |
| --- | --- |
| Cmd+A | Select all |
| Cmd+C | Copy |
| Cmd+X | Cut |
| Cmd+V | Paste |
| Cmd+Z | Undo |
| Cmd+Shift+Z | Redo |
| Cmd+F | Find |
| Cmd+G | Find next |
| Cmd+Shift+G | Find previous |
| Cmd+S | Save |
| Cmd+Shift+S | Save as |
| Cmd+P | Print |
| Cmd+W | Close window or tab |
| Cmd+Q | Quit (macOS) |
| Cmd+H | Hide app (macOS) |
| Cmd+M | Minimize window (macOS) |
| Cmd+N | New document or window |
| Cmd+O | Open |

**Navigation shortcuts**

| Shortcut | Action |
| --- | --- |
| Tab | Move focus forward |
| Shift+Tab | Move focus backward |
| Arrow keys | Move focus directionally |
| Space / Enter | Activate focused element |
| Escape | Cancel / dismiss |
| Cmd+[ | Navigate back |
| Cmd+] | Navigate forward |
| Cmd+Up Arrow | Scroll to top |
| Cmd+Down Arrow | Scroll to bottom |

#### Custom keyboard shortcuts

When defining custom shortcuts, choose modifier key combinations that don't conflict with system shortcuts. Recommended modifier hierarchy:

| Modifier | Usage |
| --- | --- |
| Cmd | Primary app actions (matches system pattern) |
| Cmd+Shift | Supplementary variants of Cmd actions |
| Cmd+Option | Alternative or less-frequent actions |
| Cmd+Control | App-specific actions unlikely to conflict |
| Cmd+Option+Shift | Reserved for rare, expert-only actions |

- **Choose mnemonics where possible.** Cmd+B for Bold, Cmd+I for Italic, and Cmd+K for Insert Link are conventional; use letter keys that relate to the action name.
- **Avoid reassigning common system shortcuts.** Cmd+C, Cmd+V, Cmd+Z, Cmd+Q, and similar system-wide shortcuts should not be overridden.
- **Limit the total number of custom shortcuts.** A large custom shortcut set is hard for users to memorize and increases the risk of conflicts.
- **Group related shortcuts under the same modifier prefix** to create a learnable pattern.
- **Document all custom shortcuts.** List them in a Help menu, in your app's documentation, and in the keyboard shortcut overlay on iPadOS.

> Tip: On Mac, you can present keyboard shortcuts in the menu bar under a Help menu or via the standard NSMenu shortcut display mechanism. On iPadOS, use UIKeyCommand.discoverabilityTitle to make shortcuts appear in the system shortcut overlay (shown when the user holds the Cmd key).

#### Platform considerations

No additional guidance for iOS, iPadOS, macOS, or tvOS beyond what is covered above.

Keyboards are not applicable to watchOS.

**visionOS**

- Virtual keyboard input should feel equivalent to physical keyboard input wherever the app supports text entry.
- Physical Bluetooth keyboards can be connected to Apple Vision Pro; support the same shortcut set you would on iPadOS.
- Use the system virtual keyboard and avoid implementing custom virtual keyboards unless your app is a keyboard app.

#### Resources

- Human Interface Guidelines: Keyboards — https://developer.apple.com/design/human-interface-guidelines/keyboards
- UIKeyCommand — https://developer.apple.com/documentation/uikit/uikeycommand
- NSMenuItem — https://developer.apple.com/documentation/appkit/nsmenuitem

#### Change log

| Date | Change |
| --- | --- |
| Jun 9, 2025 | Updated shortcut tables for macOS 26. |
| Jun 10, 2024 | Added visionOS guidance. |
| Jun 21, 2023 | Updated for iPadOS 17 keyboard shortcut overlay. |

---

## Nearby interactions

![Nearby interactions hero](https://docs-assets.developer.apple.com/published/a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7/inputs-nearby-interactions-intro@2x.png)

Source: https://developer.apple.com/design/human-interface-guidelines/nearby-interactions

Nearby Interactions uses the Ultra Wideband (UWB) chip in supported Apple devices to measure the precise distance and direction to other nearby devices with centimeter-level accuracy. Apps can use this capability for device-to-device workflows, proximity-aware features, and spatial awareness without relying on GPS or Bluetooth signal strength.

#### Best practices

- **Request Nearby Interactions permission only when your feature is clearly active.** Precision location-like capability creates privacy sensitivity; request permission in context, not at app launch.
- **Explain the benefit clearly in your permission request.** People need to understand what using Nearby Interactions enables for them — "find your friend in the crowd" is more meaningful than "use UWB for precision finding."
- **Indicate the direction and distance visually.** Use an arrow, a radar-style visualization, or proximity rings to show users where the target device is and how far away it is.
- **Update the UI in real time as distance and direction change.** The whole value of Nearby Interactions is high-frequency precision data; a UI that updates at 1 Hz or less defeats the purpose.
- **Handle the case where both devices must opt in.** Nearby Interactions requires both devices to be running compatible sessions; gracefully handle the state where the other party hasn't started the session.
- **Degrade gracefully when UWB is not available.** Older devices and some iPad models don't have UWB; fall back to Bluetooth-based proximity estimates and clearly indicate the reduced precision.

#### Device usage

- **Both devices must be unlocked and running the app (or a background session) for ranging to work.** Guide users to keep their devices active when precision finding is in progress.
- **Ranging works best when devices are held naturally — face-out, not in a pocket.** Provide a visual indicator when ranging quality degrades due to obstruction.
- **Battery usage increases during active ranging sessions.** If a ranging session is long-running, notify the user and offer a lower-frequency mode.

#### Platform considerations

No additional guidance for iPadOS.

Nearby Interactions are not applicable to macOS, tvOS, or visionOS.

**iOS**

Nearby Interactions using UWB is supported on iPhone 11 and later. Use NISession to start a session and implement NISessionDelegate to receive ranging updates.

**watchOS**

Nearby Interactions on Apple Watch use Bluetooth-based proximity rather than UWB; the precision is lower and direction is not available. Design watchOS nearby-interaction features around distance bands (near/medium/far) rather than centimeter-level precision.

#### Resources

- Human Interface Guidelines: Nearby interactions — https://developer.apple.com/design/human-interface-guidelines/nearby-interactions
- Nearby Interactions framework — https://developer.apple.com/documentation/nearbyinteraction

#### Change log

| Date | Change |
| --- | --- |
| Jun 21, 2023 | Renamed from "Ultra Wideband" to "Nearby interactions." |

---

## Pointing devices

![Pointing devices hero](https://docs-assets.developer.apple.com/published/b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7/inputs-pointing-devices-intro@2x.png)

Source: https://developer.apple.com/design/human-interface-guidelines/pointing-devices

Pointing devices — mice, trackpads, and the Apple Pencil in pointer mode — let people interact with apps using a cursor or pointer. On Mac, pointer input is the primary interaction modality. On iPad, pointer support enhances precision for users who have connected a Magic Keyboard, Magic Trackpad, or mouse. On visionOS, indirect eye-and-pinch input has pointer-like semantics.

#### Best practices

- **Make all interactive elements reachable by pointer.** Every button, link, and control should be activatable with a click or tap from a pointer device.
- **Use pointer shape and effect changes to communicate interactivity.** Change the cursor to a pointing hand over links and interactive elements; use text insertion cursor over editable text.
- **Design hover states for all interactive elements.** On Mac and iPad with pointer, users frequently hover before clicking; use hover to preview the action, highlight the target, or reveal secondary information.
- **Support right-click and secondary click for context menus.** On Mac, right-click is a primary way to access contextual actions; on iPad, secondary click is increasingly common with external trackpads.
- **Support scroll wheel and trackpad scrolling everywhere content overflows.** Do not require clicking on a scrollbar; scroll events should work anywhere the pointer is within a scrollable region.

#### Platform considerations

No additional guidance for iOS beyond what is covered above.

Pointing devices are not applicable to tvOS or watchOS.

**iPadOS**

When a pointer device is connected on iPad, the system pointer appears and the interaction model shifts from touch to pointer-based.

- **Adapt your layout for pointer interaction.** Smaller hit targets that are fine for touch may feel imprecise with a pointer; increase hit target sizes for primary actions.
- **Implement pointer interaction APIs.** Use UIPointerInteraction and UIPointerStyle to customize pointer appearance and behavior for your controls.
- **Support drag and drop from pointer.** Pointer drag interactions are expected to work with the standard drag-and-drop APIs.

**Pointer shape and content effects**

The pointer changes shape to communicate what will happen when the user clicks. Standard pointer shapes:

- Default arrow — navigation, general pointing
- I-beam — editable text
- Pointing hand — links and tappable items
- Crosshair — drawing, selection rectangles
- Resize arrows — resizable edges and handles

Content effects change the interaction appearance beyond the cursor:

- **Highlight effect** — pointer blends into the element (used for buttons and controls)
- **Lift effect** — pointer disappears and the element scales up slightly (used for icons and images)
- **Hover effect** — custom appearance change on hover without pointer integration

**Pointer accessories**

Pointer accessories extend the pointer with supplemental indicators:

- Attach small indicators to the pointer to show secondary state (e.g., a "+" badge for add actions, an arrow for drag directions).
- Keep accessories small and clearly distinct from the pointer itself.
- Use accessories sparingly; overuse dilutes their communicative value.

**Pointer magnetism**

Pointer magnetism snaps the pointer to an interactive element when it moves nearby, reducing the motor demand for hitting small targets.

- Enable pointer magnetism for small controls like toolbar buttons and segmented control segments.
- Tune the magnetism radius so it feels helpful rather than resistive — too large a radius causes unintended snapping.
- Pair magnetism with a hover effect so users can see when magnetism is active.
- Test magnetism with diverse users, as some people find aggressive magnetism disorienting.

**Standard pointers and effects**

The system provides standard pointer styles for common control types. Prefer these over custom styles:

- UIPointerStyle.automatic — system chooses the appropriate style
- UIPointerStyle.cursor — custom cursor shape
- UIPointerStyle.hidden — hide the pointer (for immersive experiences)
- UIPointerStyle.system — use the default system pointer

Standard effects:
- UIPointerEffect.highlight — highlight the element
- UIPointerEffect.lift — lift the element
- UIPointerEffect.hover — custom hover effect

**Customizing pointers**

- Animate pointer shape changes smoothly; abrupt shape changes are jarring.
- Maintain consistent pointer behavior across your app; don't change the pointer shape for similar controls in different parts of the app.
- Use system-provided pointer shapes rather than fully custom cursors whenever they communicate the right semantic.
- Provide custom pointer shapes only for genuinely novel interaction types where no standard shape applies.
- Ensure custom pointer shapes are legible at both normal and retina resolutions.
- Test custom pointers in both Light Mode and Dark Mode.
- Avoid animated pointers except for loading states.

**macOS**

macOS supports a rich set of trackpad gestures beyond basic pointing:

| Gesture | Action |
| --- | --- |
| Two-finger swipe left/right | Page navigation / swipe back/forward |
| Two-finger scroll | Scroll content |
| Pinch/spread | Zoom in/out (in supporting apps) |
| Two-finger rotate | Rotate content (in supporting apps) |
| Two-finger double tap | Smart zoom |
| Three-finger swipe up | Mission Control |
| Three-finger swipe down | App Expose |
| Four-finger swipe left/right | Switch between full-screen apps and Spaces |
| Force click | Quick Look / data detector lookup |
| Tap to click | Click without pressing |

**Pointers**

Standard macOS cursor/pointer types:

| Pointer | Use |
| --- | --- |
| Arrow | Default; general navigation |
| I-beam | Text editing |
| Crosshair | Drawing selection rectangles, pixel-level operations |
| Open hand | Draggable content that can be grabbed |
| Closed hand | Actively dragging content |
| Pointing hand | Links and clickable items |
| Resize horizontal | Horizontal resize handle |
| Resize vertical | Vertical resize handle |
| Resize diagonal (NW/SE) | Diagonal resize handle |
| Resize diagonal (NE/SW) | Diagonal resize handle |
| Zoom in | Click to zoom in |
| Zoom out | Click to zoom out |
| Forbidden | Action not allowed |
| Spinning beach ball | System is busy (do not use; managed by the OS) |
| Disappearing item | Drag-to-trash |
| Context menu | Secondary click available |
| Copy | Drag-to-copy operation in progress |
| Alias | Creating an alias |
| Operation not allowed | Operation not permitted on this target |

**visionOS**

- Indirect pointing in visionOS is driven by eye tracking combined with pinch gestures; no physical pointer device is required.
- Physical Magic Trackpad and mouse are supported; pointer input then works similarly to iPadOS.
- Use pointer interaction APIs consistently between trackpad-driven and eye-driven input.

#### Resources

- Human Interface Guidelines: Pointing devices — https://developer.apple.com/design/human-interface-guidelines/pointing-devices
- UIPointerInteraction — https://developer.apple.com/documentation/uikit/uipointerinteraction
- NSCursor — https://developer.apple.com/documentation/appkit/nscursor

#### Change log

| Date | Change |
| --- | --- |
| Jun 21, 2023 | Updated for iPadOS 17 pointer improvements. |

---

## Remotes

![Remotes hero](https://docs-assets.developer.apple.com/published/c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7/inputs-remotes-intro@2x.png)

Source: https://developer.apple.com/design/human-interface-guidelines/remotes

The Siri Remote is the primary input device for Apple TV. It combines a touch surface, accelerometer, gyroscope, and buttons into a unified controller. Compatible Bluetooth remotes from third parties are also supported. Apps should design their interaction model around the remote as the primary input and never require a secondary device.

#### Best practices

- **Make all interactions achievable with the Siri Remote.** Never require a touch device, keyboard, or game controller to complete any action in your app.
- **Use the touch surface for fine-grained directional input.** The touch surface supports swipe gestures for scrolling, and touch-and-hold for initiating drag operations.
- **Map the Menu button to navigation back.** People expect the Menu button to go back one screen; always implement this behavior.
- **Map the Play/Pause button to media playback control.** In any screen that contains playable media, Play/Pause should toggle between play and pause states.
- **Support the TV button behavior.** A single press returns to the Apple TV home screen; don't intercept this button.
- **Don't require precise tapping on small targets.** The remote touch surface has limited resolution for cursor positioning; design for focus-based navigation rather than free-form pointer placement.
- **Use the accelerometer and gyroscope only for supplemental interaction.** Motion-based remote controls can add to a gaming experience but should never be required for navigation or key actions.
- **Support both Siri Remote generations.** The second-generation Siri Remote added a clickable touch surface ring and a mute button; ensure your app's button mapping works with both generations.

#### Gestures

- **Swipe on the touch surface to move focus directionally.** The standard remote swipe gesture moves focus to the next focusable element in the swipe direction.
- **Click the touch surface center to activate the focused element.** This is the Select action and is analogous to a tap in touch-based UIs.
- **Swipe up from the touch surface during video playback to reveal info overlays.** This is a system-standard gesture; don't override it in video playback contexts.

#### Buttons

| Button | Standard action |
| --- | --- |
| Menu | Navigate back; long press returns to home screen |
| Play/Pause | Toggle media playback |
| Siri / Search | Invoke Siri or open search |
| TV / Home | Single press goes to Apple TV home; double press opens App Switcher |

#### Compatible remotes

- Third-party Bluetooth remotes that implement the standard HID remote profile are automatically compatible with Apple TV apps.
- Game controllers connected to Apple TV can also serve as remotes for app navigation using their D-pad and face buttons.
- Design your focus-based navigation to work correctly with any directional input device, not just the Siri Remote.

#### Platform considerations

Remotes are specific to Apple TV (tvOS). Not applicable to iOS, iPadOS, macOS, visionOS, or watchOS.

#### Resources

- Human Interface Guidelines: Remotes — https://developer.apple.com/design/human-interface-guidelines/remotes
- GCController — https://developer.apple.com/documentation/gamecontroller/gccontroller
