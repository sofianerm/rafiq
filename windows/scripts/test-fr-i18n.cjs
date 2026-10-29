const {readFileSync} = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const assert = require('node:assert/strict');
const {test} = require('node:test');

// Exercise the page's actual lookup and formatters without a WebView or Tauri.
const html = readFileSync(path.join(__dirname, '../rafiq/ui/notch.html'), 'utf8');
const source = html.slice(html.indexOf("let uiLang='en';"), html.indexOf('function setUiLanguage'));
function card(lang) {
  return vm.runInNewContext(source + `; uiLang=${JSON.stringify(lang)}; ({textCopy, ui:ui()})`);
}

test('the French card keeps numbers, plan names and unknown vendor messages', () => {
  const {textCopy, ui} = card('fr');
  assert.equal(textCopy('Current session'), 'Session en cours');
  assert.equal(textCopy('Sign in'), 'Se connecter');
  assert.equal(textCopy('Rate limited — retrying in 30s'), 'Trop de requêtes — nouvel essai dans 30 s');
  assert.equal(textCopy('5h limit'), 'Limite de 5 h');
  assert.equal(textCopy('Unlimited on the Pro plan — nothing to meter'), 'Illimité avec le forfait Pro — rien à mesurer');
  assert.equal(textCopy('Working in example-repo'), 'Au travail dans example-repo');
  assert.equal(textCopy('New vendor message'), 'New vendor message');
  assert.equal(ui.usedLeft('0.6', '99.4'), '0.6 % utilisé · 99.4 % restant');
  assert.equal(ui.resetsIn(51), 'Réinitialisé dans 51 min');
  assert.equal(ui.ago(120), 'il y a 2 h');
  assert.equal(ui.andMore(3), 'et 3 de plus');
});

test('an unsupported card language still uses English', () => {
  for (const lang of ['xx', 'ko', 'ru']) {
    const {textCopy, ui} = card(lang);
    assert.equal(textCopy('Current session'), 'Current session');
    assert.equal(ui.resetsIn(51), 'Resets in 51 min');
    assert.equal(ui.updated('20m ago'), 'Updated 20m ago');
  }
});
