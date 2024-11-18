use pommes::{Dependencies, Dependency, Project};
use std::io::ErrorKind;
use std::{fs, io, path::Path};

pub struct MavenDependency(String);

impl From<&Dependency> for MavenDependency {
    fn from(value: &Dependency) -> Self {
        MavenDependency(format!(
            "{}:{}:{}",
            value.group_id,
            value.artifact_id,
            value.version.clone().expect("no version")
        ))
    }
}

impl MavenDependency {
    pub fn write<P: AsRef<Path>>(
        project: Project,
        path: P,
        dependencies: Vec<Self>,
    ) -> io::Result<()> {
        let maven_dependencies = dependencies
            .into_iter()
            .map(|d| d.0)
            .collect::<Vec<String>>();

        let mut content = String::new();

        for (i, depend) in maven_dependencies.iter().enumerate() {
            content.push_str(
                &depend
                    .replace("${project.version}", &project.version.clone().unwrap())
                    .replace(
                        "${minecraft.version}",
                        &project
                            .properties
                            .get("minecraft_version")
                            .unwrap()
                            .replace("_", ".")
                            .replace("R", ""),
                    ),
            );

            if i != maven_dependencies.len() - 1 {
                content.push('\n');
            }
        }

        fs::write(path, content)
    }
}

pub fn read_dependencies<P: AsRef<Path>>(
    pom_path: P,
) -> io::Result<(Project, Vec<MavenDependency>)> {
    let contents = fs::read_to_string(pom_path)?;
    let project = serde_xml_rs::from_str::<Project>(&contents).map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;

    let dependencies = project.dependencies
        .as_ref()
        .unwrap_or(&Dependencies { dependencies: vec![] })
        .dependencies
        .clone();

    Ok((
        project,
        dependencies
            .iter()
            .filter(|dep| dep.scope.is_none() || dep.scope == Some("compile".to_string()))
            .filter(|dep| dep.artifact_id != "minecraft-server")
            .map(MavenDependency::from)
            .collect(),
    ))
}