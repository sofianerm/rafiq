// L'essai gratuit de Clippy : quelques réécritures offertes avant de mettre sa propre clé d'API.
//
// POST /v1/clippy  {install_id, prompt, text}  →  {output, remaining}
// GET  /v1/clippy?install_id=…                 →  {remaining}
//
// La clé DeepSeek est un secret Cloudflare (DEEPSEEK_API_KEY), jamais dans le code. Trois plafonds,
// comptés dans KV : par installation (à vie), par IP et par jour, et pour tout le monde par jour.
// Rien du texte ni de la réponse n'est journalisé ni gardé : seuls des compteurs sont écrits.

const DAY = 86400;

const json = (body, status = 200) =>
  new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json; charset=utf-8' } });

const today = () => new Date().toISOString().slice(0, 10);

async function count(env, key) {
  return Number((await env.TRIAL.get(key)) || 0);
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (url.pathname !== '/v1/clippy') return json({ error: 'not found' }, 404);

    const perInstall = Number(env.PER_INSTALL || 5);
    const perIpDay = Number(env.PER_IP_DAY || 10);
    const globalDay = Number(env.GLOBAL_DAY || 200);

    if (request.method === 'GET') {
      const id = url.searchParams.get('install_id') || '';
      if (!/^[a-f0-9-]{16,64}$/.test(id)) return json({ error: 'install_id invalide' }, 400);
      return json({ remaining: Math.max(0, perInstall - (await count(env, `i:${id}`))) });
    }
    if (request.method !== 'POST') return json({ error: 'méthode non permise' }, 405);

    let body;
    try { body = await request.json(); } catch { return json({ error: 'JSON invalide' }, 400); }
    const id = String(body.install_id || '');
    const prompt = String(body.prompt || '');
    const text = String(body.text || '');
    if (!/^[a-f0-9-]{16,64}$/.test(id)) return json({ error: 'install_id invalide' }, 400);
    if (!text.trim()) return json({ error: 'texte vide' }, 400);
    if (text.length > Number(env.MAX_CHARS || 4000) || prompt.length > 2000) {
      return json({ error: 'Texte trop long pour l’essai gratuit' }, 413);
    }

    const ip = request.headers.get('CF-Connecting-IP') || 'unknown';
    const day = today();
    const kInstall = `i:${id}`, kIp = `ip:${ip}:${day}`, kGlobal = `g:${day}`;
    const [used, ipUsed, globalUsed] = await Promise.all([count(env, kInstall), count(env, kIp), count(env, kGlobal)]);
    if (used >= perInstall) {
      return json({ error: 'Tes essais gratuits sont utilisés : ajoute ta propre clé d’API dans les réglages de Clippy.', remaining: 0 }, 429);
    }
    if (ipUsed >= perIpDay || globalUsed >= globalDay) {
      return json({ error: 'L’essai gratuit est saturé pour aujourd’hui : réessaie demain, ou ajoute ta propre clé d’API.', remaining: perInstall - used }, 429);
    }

    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), 40000);
    let output;
    try {
      const resp = await fetch('https://api.deepseek.com/chat/completions', {
        method: 'POST',
        signal: controller.signal,
        headers: { 'content-type': 'application/json', authorization: `Bearer ${env.DEEPSEEK_API_KEY}` },
        body: JSON.stringify({
          model: env.MODEL || 'deepseek-v4-pro',
          messages: [{ role: 'system', content: prompt }, { role: 'user', content: text }],
          temperature: 0.4,
          max_tokens: 2000,
          stream: false,
        }),
      });
      if (!resp.ok) return json({ error: `L’IA de l’essai ne répond pas (${resp.status}), réessaie plus tard.` }, 502);
      const data = await resp.json();
      output = String(data?.choices?.[0]?.message?.content || '').trim();
    } catch {
      return json({ error: 'L’IA de l’essai ne répond pas, réessaie plus tard.' }, 504);
    } finally {
      clearTimeout(timer);
    }
    if (!output) return json({ error: 'L’IA n’a rien renvoyé' }, 502);

    // Compté seulement quand l'essai a vraiment servi
    await Promise.all([
      env.TRIAL.put(kInstall, String(used + 1)),
      env.TRIAL.put(kIp, String(ipUsed + 1), { expirationTtl: 2 * DAY }),
      env.TRIAL.put(kGlobal, String(globalUsed + 1), { expirationTtl: 2 * DAY }),
    ]);
    return json({ output, remaining: Math.max(0, perInstall - used - 1) });
  },
};
