use crate::bundlers::Bundler;
use crate::{BundleOptions, Target};
use ndk_build::apk::{ApkConfig, StripConfig};
use ndk_build::cargo::VersionCode;
use ndk_build::manifest::{
    Activity, AndroidManifest, Application, IntentFilter, MetaData, Permission,
};
use ndk_build::ndk::Ndk;
use ndk_build::target::Target as AndroidTarget;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AndroidBundler {}

impl AndroidBundler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Bundler for AndroidBundler {
    fn bundle(&self, options: &BundleOptions) {
        let bundle_location = options.bundle_location();
        let app_name = options.app_name();

        let resources_dir = bundle_location.join("res");
        if resources_dir.exists() {
            std::fs::remove_dir_all(resources_dir.as_path()).unwrap();
        }
        std::fs::create_dir_all(resources_dir.as_path()).unwrap();

        for each in options.icons() {
            let mut copy_options = fs_extra::dir::CopyOptions::default();
            copy_options.content_only = true;
            fs_extra::dir::copy(each, resources_dir.as_path(), &copy_options).unwrap();
        }
        let icon = if options.icons().is_empty() {
            None
        } else {
            Some("@mipmap/ic_launcher".to_string())
        };

        let android_target = match options.target() {
            Target::AArch64LinuxAndroid => AndroidTarget::Arm64V8a,
            _ => {
                panic!(
                    "Unsupported android target: {}",
                    options.target().to_string()
                )
            }
        };

        let android_activity = Activity {
            config_changes: Some("orientation|keyboardHidden|screenSize".to_string()),
            label: Some(app_name.to_string()),
            launch_mode: None,
            name: "android.app.NativeActivity".to_string(),
            orientation: None,
            exported: None,
            resizeable_activity: None,
            meta_data: vec![MetaData {
                name: "android.app.lib_name".to_string(),
                value: "vm_client_android".to_string(),
            }],
            intent_filter: vec![IntentFilter {
                actions: vec!["android.intent.action.MAIN".to_string()],
                categories: vec!["android.intent.category.LAUNCHER".to_string()],
                data: vec![],
            }],
        };

        let android_application = Application {
            debuggable: Some(true),
            theme: Some("@android:style/Theme.DeviceDefault.NoActionBar.Fullscreen".to_string()),
            has_code: false,
            icon,
            label: app_name.to_string(),
            meta_data: vec![],
            activity: android_activity,
        };

        let mut manifest = AndroidManifest::default();
        manifest.package = options.identifier().to_string();
        manifest.application = android_application;
        manifest.version_name = Some(options.version().to_string());
        manifest.version_code = Some(
            VersionCode::from_semver(options.version().to_string().as_str())
                .unwrap()
                .to_code(1),
        );
        manifest.sdk.min_sdk_version = Some(30);
        manifest.sdk.target_sdk_version = Some(30);
        manifest.sdk.max_sdk_version = Some(33);
        manifest.uses_permission = vec![
            Permission {
                name: "android.permission.INTERNET".to_string(),
                max_sdk_version: None,
            },
            Permission {
                name: "android.permission.ACCESS_NETWORK_STATE".to_string(),
                max_sdk_version: None,
            },
        ];

        let ndk = Ndk::from_env().unwrap();
        let config = ApkConfig {
            ndk: ndk.clone(),
            build_dir: bundle_location.clone(),
            apk_name: app_name.to_string(),
            assets: None,
            resources: Some(resources_dir),
            manifest,
            disable_aapt_compression: !options.release(),
            strip: StripConfig::Default,
            reverse_port_forward: Default::default(),
        };

        let mut apk = config.create_apk().expect("Create APK");
        let lib_search_path = self.compiled_libraries_directory(options);

        self.compiled_libraries(options)
            .iter()
            .for_each(|compiled_library_path| {
                apk.add_lib_recursively(
                    &compiled_library_path,
                    android_target,
                    &[lib_search_path.as_path()],
                )
                .expect("Add runtime lib")
            });

        apk.add_pending_libs_and_align()
            .expect("Add pending libs and align");
    }

    fn bundled_executable_directory(&self, options: &BundleOptions) -> PathBuf {
        options
            .bundle_location()
            .join(options.app_name())
            .join("lib")
    }

    fn bundled_resources_directory(&self, options: &BundleOptions) -> PathBuf {
        options
            .bundle_location()
            .join(options.app_name())
            .join("assets")
    }

    fn clone_bundler(&self) -> Box<dyn Bundler> {
        Box::new(Clone::clone(self))
    }
}
