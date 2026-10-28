// Parses the inline <script> of every page in rafiq/ui without running it.
//
// A syntax error anywhere in a page's script stops all of it, and the window
// opens empty with nothing in the log to say why. cargo never reads these
// files, so the Rust build stays green through it: a conflict resolution that
// left a stray function body in settings.html shipped that way in 1.12.0.
import { readdirSync, readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import vm from 'node:vm';

const ui = join(dirname(fileURLToPath(import.meta.url)), '..', 'rafiq', 'ui');
let failed = false;

for (const name of readdirSync(ui).filter(f => f.endsWith('.html'))) {
  const html = readFileSync(join(ui, name), 'utf8');
  const blocks = [...html.matchAll(/<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/g)];
  blocks.forEach((block, index) => {
    // The line the block starts on, so an error points into the .html itself.
    const lineOffset = html.slice(0, block.index).split('\n').length - 1;
    try {
      // `type="module"` pages would need vm.SourceTextModule; none are modules
      // today, and compiling as a script is what WebView2 does with them.
      new vm.Script(block[1], { filename: `${name} <script> #${index + 1}`, lineOffset });
    } catch (error) {
      failed = true;
      console.error(`${name}: ${error.message}`);
      if (error.stack) console.error(error.stack.split('\n').slice(0, 3).join('\n'));
    }
  });
  console.log(`${name}: ${blocks.length} inline script(s) checked`);
}

// The notch's modules are separate files, loaded by <script src>: the same failure, one file at a time.
const modules = join(ui, 'modules');
for (const name of readdirSync(modules).filter(f => f.endsWith('.js'))) {
  try {
    new vm.Script(readFileSync(join(modules, name), 'utf8'), { filename: `modules/${name}` });
  } catch (error) {
    failed = true;
    console.error(`modules/${name}: ${error.message}`);
  }
  console.log(`modules/${name}: checked`);
}

// Settings loads the same module files to list them; a module missing there could not be hidden or moved.
const srcs = page => [...readFileSync(join(ui, page), 'utf8').matchAll(/<script src="(modules\/[^"]+)"/g)].map(m => m[1]).join(',');
if (srcs('notch.html') !== srcs('settings.html')) {
  failed = true;
  console.error('notch.html and settings.html must load the same modules/*.js, in the same order');
} else {
  console.log('module <script src> lists match in notch.html and settings.html');
}

process.exit(failed ? 1 : 0);
