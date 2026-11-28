//! Cascading `<MonoxusConfig>` context providing inheritable defaults for
//! reading direction (`dir`), overlay portal host (`default_portal_to`), and
//! calendar/date-time locale (`default_locale`) (`S2-LD-07`).

use dioxus::prelude::*;
use ui_sdk::foundation::{
    date_time::CalendarLocale, overlay::portal::PortalHost, shared::Direction,
};

use super::{shared::DirectionContext, state::sync_signal};

/// Reactive context provided by [`MonoxusConfig`] (and updated by
/// [`super::shared::DirectionProvider`]) so nested providers inherit omitted
/// fields and nearest-ancestor direction always wins.
#[derive(Clone, Copy)]
pub struct MonoxusConfigContext {
    pub dir: Signal<Option<Direction>>,
    pub default_portal_to: Signal<Option<PortalHost>>,
    pub default_locale: Signal<Option<CalendarLocale>>,
}

/// Snapshot of resolved [`MonoxusConfig`] values returned by [`use_monoxus_config`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MonoxusConfigValues {
    pub dir: Option<Direction>,
    pub default_portal_to: Option<PortalHost>,
    pub default_locale: Option<CalendarLocale>,
}

/// Props for [`MonoxusConfig`].
#[derive(Clone, PartialEq, Props)]
pub struct MonoxusConfigProps {
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub default_portal_to: Option<PortalHost>,
    #[props(default)]
    pub default_locale: Option<CalendarLocale>,
    pub children: Element,
}

/// Zero-wrapper-DOM cascading configuration provider.
///
/// Merges each field with any ancestor [`MonoxusConfigContext`] via
/// `own.or(parent)`, and synchronizes [`DirectionContext`] so interleaving
/// `<MonoxusConfig>` and `<DirectionProvider>` always resolves the nearest
/// ancestor.
#[component]
pub fn MonoxusConfig(props: MonoxusConfigProps) -> Element {
    let parent_config = use_hook(|| try_consume_context::<MonoxusConfigContext>());
    let parent_dir_ctx = use_hook(|| try_consume_context::<DirectionContext>());

    let resolved_dir = props
        .dir
        .or_else(|| parent_config.and_then(|ctx| (ctx.dir)()))
        .or_else(|| parent_dir_ctx.map(|ctx| (ctx.0)()));
    let resolved_portal = props
        .default_portal_to
        .or_else(|| parent_config.and_then(|ctx| (ctx.default_portal_to)()));
    let resolved_locale = props
        .default_locale
        .or_else(|| parent_config.and_then(|ctx| (ctx.default_locale)()));

    let mut dir_sig = use_signal(|| resolved_dir);
    sync_signal(&mut dir_sig, resolved_dir);

    let mut portal_sig = use_signal(|| resolved_portal.clone());
    sync_signal(&mut portal_sig, resolved_portal);

    let mut locale_sig = use_signal(|| resolved_locale.clone());
    sync_signal(&mut locale_sig, resolved_locale);

    let effective_dir = resolved_dir.unwrap_or(Direction::Ltr);
    let mut dir_ctx_sig = use_signal(|| effective_dir);
    sync_signal(&mut dir_ctx_sig, effective_dir);

    use_context_provider(|| MonoxusConfigContext {
        dir: dir_sig,
        default_portal_to: portal_sig,
        default_locale: locale_sig,
    });
    use_context_provider(|| DirectionContext(dir_ctx_sig));

    props.children
}

/// Reads the nearest [`MonoxusConfigContext`] values, returning all `None` when
/// no provider is mounted above the caller.
pub fn use_monoxus_config() -> MonoxusConfigValues {
    match try_consume_context::<MonoxusConfigContext>() {
        Some(ctx) => MonoxusConfigValues {
            dir: (ctx.dir)(),
            default_portal_to: (ctx.default_portal_to)(),
            default_locale: (ctx.default_locale)(),
        },
        None => MonoxusConfigValues::default(),
    }
}

/// Resolves the effective [`PortalHost`] using `override_host`, then the
/// nearest [`MonoxusConfigContext::default_portal_to`], then `fallback`.
pub fn use_portal_host_context(
    override_host: Option<PortalHost>,
    fallback: PortalHost,
) -> PortalHost {
    let inherited = try_consume_context::<MonoxusConfigContext>()
        .and_then(|ctx| (ctx.default_portal_to)());
    override_host.or(inherited).unwrap_or(fallback)
}

/// Resolves the effective [`CalendarLocale`] using `override_locale`, then the
/// nearest [`MonoxusConfigContext::default_locale`], then
/// [`CalendarLocale::default`] (`EnUs`).
pub fn use_locale_context(override_locale: Option<CalendarLocale>) -> CalendarLocale {
    let inherited =
        try_consume_context::<MonoxusConfigContext>().and_then(|ctx| (ctx.default_locale)());
    override_locale.or(inherited).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use dioxus::prelude::*;
    use ui_sdk::foundation::{
        date_time::CalendarLocale, overlay::portal::PortalHost, shared::Direction,
    };

    use super::{
        MonoxusConfig, MonoxusConfigValues, use_locale_context, use_monoxus_config,
        use_portal_host_context,
    };
    use crate::foundation::shared::{DirectionProvider, use_direction_context};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Snapshot {
        config: MonoxusConfigValues,
        dir: Direction,
        portal: PortalHost,
        portal_override: PortalHost,
        locale: CalendarLocale,
        locale_override: CalendarLocale,
    }

    #[component]
    fn Probe(out: Rc<RefCell<Option<Snapshot>>>) -> Element {
        let snap = Snapshot {
            config: use_monoxus_config(),
            dir: use_direction_context(None),
            portal: use_portal_host_context(None, PortalHost::default_host()),
            portal_override: use_portal_host_context(
                Some(PortalHost::inline()),
                PortalHost::default_host(),
            ),
            locale: use_locale_context(None),
            locale_override: use_locale_context(Some(CalendarLocale::EnUs)),
        };
        *out.borrow_mut() = Some(snap);
        rsx! { span { "probe" } }
    }

    #[test]
    fn nested_config_inherits_omitted_fields_and_honors_overrides() {
        let out: Rc<RefCell<Option<Snapshot>>> = Rc::new(RefCell::new(None));

        #[component]
        fn App(out: Rc<RefCell<Option<Snapshot>>>) -> Element {
            rsx! {
                MonoxusConfig {
                    dir: Direction::Rtl,
                    default_portal_to: PortalHost::named("app-portal"),
                    default_locale: CalendarLocale::KoKr,
                    MonoxusConfig {
                        default_portal_to: PortalHost::named("modal-portal"),
                        Probe { out }
                    }
                }
            }
        }

        let mut dom = VirtualDom::new_with_props(App, AppProps { out: Rc::clone(&out) });
        dom.rebuild_in_place();

        let snap = out.borrow().clone().expect("probe snapshot");
        assert_eq!(snap.dir, Direction::Rtl);
        assert_eq!(snap.config.dir, Some(Direction::Rtl));
        assert_eq!(snap.portal, PortalHost::named("modal-portal"));
        assert_eq!(snap.portal_override, PortalHost::inline());
        assert_eq!(snap.locale, CalendarLocale::KoKr);
        assert_eq!(snap.locale_override, CalendarLocale::EnUs);
    }

    #[test]
    fn interleaved_direction_provider_and_monoxus_config_resolves_nearest_ancestor() {
        let out_inner: Rc<RefCell<Option<Snapshot>>> = Rc::new(RefCell::new(None));

        #[component]
        fn InterleavedApp(out_inner: Rc<RefCell<Option<Snapshot>>>) -> Element {
            rsx! {
                MonoxusConfig {
                    dir: Direction::Rtl,
                    default_portal_to: PortalHost::named("outer-portal"),
                    default_locale: CalendarLocale::KoKr,
                    DirectionProvider {
                        dir: Direction::Ltr,
                        MonoxusConfig {
                            default_locale: CalendarLocale::EnGb,
                            Probe { out: out_inner }
                        }
                    }
                }
            }
        }

        let mut dom = VirtualDom::new_with_props(
            InterleavedApp,
            InterleavedAppProps {
                out_inner: Rc::clone(&out_inner),
            },
        );
        dom.rebuild_in_place();

        let snap = out_inner.borrow().clone().expect("inner snapshot");
        // DirectionProvider(Ltr) overrode outer MonoxusConfig(Rtl) while preserving portal host,
        // and inner MonoxusConfig preserved Ltr + outer-portal while overriding locale to EnGb.
        assert_eq!(snap.dir, Direction::Ltr);
        assert_eq!(snap.portal, PortalHost::named("outer-portal"));
        assert_eq!(snap.locale, CalendarLocale::EnGb);
    }
}
