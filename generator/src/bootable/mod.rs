use std::io::{self, Write};

use bootspec::SpecialisationName;

use crate::{Generation, Result};

mod efi;
mod toplevel;

pub use efi::EfiProgram;
pub use toplevel::BootableToplevel;

pub enum Bootable {
    Linux(BootableToplevel),
    Efi(EfiProgram),
}

/// `flatten` takes in a list of [`Generation`]s and returns a list of [`BootableToplevel`]s by:
///
/// 1. transforming each [`Generation`] into a [`BootableToplevel`]; and
/// 2. recursing into each [`Generation`]s specialisations (if any) and transforming them into
///    [`BootableToplevel`]s of their own (and so on and so forth).
///
/// This makes it easy to create boot entries for all possible [`BootableToplevel`]s (both the
/// "system profile" as well as its many possible specialisations), while also ensuring we encounter
/// potential infinite recursion as early as possible.
pub fn flatten(inputs: Vec<Generation>) -> Result<Vec<BootableToplevel>> {
    self::flatten_impl(inputs, None)
}

fn flatten_impl(
    inputs: Vec<Generation>,
    specialisation_name: Option<SpecialisationName>,
) -> Result<Vec<BootableToplevel>> {
    let mut toplevels = Vec::new();

    for input in inputs {
        let generation_index = input.index;
        let profile_name = input.profile.clone();
        let input: bootspec::v1::GenerationV1 = match input.bootspec.generation {
            bootspec::generation::Generation::V1(input) => input,
            _ => panic!("unsupported generation version!"),
        };
        let toplevel = input.bootspec.toplevel.clone();

        toplevels.push(BootableToplevel {
            label: input.bootspec.label,
            kernel: input.bootspec.kernel,
            kernel_params: input.bootspec.kernel_params,
            init: input.bootspec.init,
            initrd: input.bootspec.initrd,
            toplevel,
            specialisation_name: specialisation_name.clone(),
            generation_index,
            profile_name: profile_name.clone(),
        });

        for (name, desc) in input.specialisations {
            writeln!(
                io::stderr(),
                "Flattening specialisation '{name}' of toplevel {toplevel}: {path}",
                toplevel = input.bootspec.toplevel.0.display(),
                name = name.0,
                path = desc.bootspec.toplevel.0.display()
            )?;

            let generation = Generation {
                index: generation_index,
                profile: profile_name.clone(),
                bootspec: bootspec::BootJson {
                    generation: bootspec::generation::Generation::V1(desc),
                    extensions: Default::default(),
                },
            };

            toplevels.extend(self::flatten_impl(vec![generation], Some(name))?);
        }
    }

    Ok(toplevels)
}
