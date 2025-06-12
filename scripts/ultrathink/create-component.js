#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

function createComponent(name) {
    if (!name) {
        console.error('Please provide a component name');
        process.exit(1);
    }
    
    // Convert to PascalCase
    const componentName = name.charAt(0).toUpperCase() + name.slice(1);
    const fileName = name.toLowerCase();
    
    // Component template
    const componentCode = `use layer9::prelude::*;

/// ${componentName} component
#[component]
pub fn ${componentName}(props: ${componentName}Props) -> Element {
    let state = use_state(|| ${componentName}State::default());
    
    view! {
        <div class="layer9-${fileName}">
            <h2>{props.title}</h2>
            {props.children}
        </div>
    }
}

#[derive(Props, PartialEq)]
pub struct ${componentName}Props {
    #[props(default = "${componentName}")]
    pub title: &'static str,
    
    #[props(optional)]
    pub children: Children,
}

#[derive(Default, Clone)]
struct ${componentName}State {
    // Add state fields here
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_${fileName}_renders() {
        let component = ${componentName}(${componentName}Props {
            title: "Test",
            children: None,
        });
        
        assert!(component.is_some());
    }
}`;

    // Determine output path
    const componentsDir = path.join(process.cwd(), 'crates', 'core', 'src', 'components');
    
    // Create components directory if it doesn't exist
    if (!fs.existsSync(componentsDir)) {
        fs.mkdirSync(componentsDir, { recursive: true });
    }
    
    const filePath = path.join(componentsDir, `${fileName}.rs`);
    
    // Check if file already exists
    if (fs.existsSync(filePath)) {
        console.error(`Component ${componentName} already exists at ${filePath}`);
        process.exit(1);
    }
    
    // Write component file
    fs.writeFileSync(filePath, componentCode);
    
    console.log(`✅ Created component: ${componentName}`);
    console.log(`📁 Location: ${filePath}`);
    
    // Update mod.rs to export the new component
    const modPath = path.join(componentsDir, 'mod.rs');
    if (fs.existsSync(modPath)) {
        const modContent = fs.readFileSync(modPath, 'utf8');
        const newExport = `pub mod ${fileName};\npub use ${fileName}::${componentName};\n`;
        fs.writeFileSync(modPath, modContent + '\n' + newExport);
    } else {
        fs.writeFileSync(modPath, `pub mod ${fileName};\npub use ${fileName}::${componentName};\n`);
    }
    
    console.log(`📝 Updated module exports`);
    console.log(`\n🎉 Component ${componentName} is ready to use!`);
}

// Main execution
const componentName = process.argv[2];
createComponent(componentName);