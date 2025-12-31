//! Spanish language patterns for query classification
//!
//! This module contains regex patterns in Spanish for each query category.

use crate::patterns::WeightedPattern;

/// Build Spanish math patterns
pub fn build_math_patterns_es() -> Vec<WeightedPattern> {
    vec![
        // Operaciones matemáticas - alta prioridad
        WeightedPattern::new(r"(?i)\b\d+\s*[\+\-\*\/\^]\s*\d+", 1.5),
        WeightedPattern::new(r"(?i)\bcu[aá]nto\s+(es|son|vale|da)\b", 1.5),
        WeightedPattern::new(r"(?i)\bcalcul(a|ar|e|o)\b", 1.0),
        WeightedPattern::new(r"(?i)\bresuelv(e|a|er|o)\b", 1.0),
        WeightedPattern::new(r"(?i)\bresolver\b", 1.0),
        
        // Términos matemáticos
        WeightedPattern::new(r"(?i)\becuaci[oó]n(es)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bmatem[aá]tica(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bf[oó]rmula(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\b[aá]lgebra\b", 1.0),
        WeightedPattern::new(r"(?i)\bgeometr[ií]a\b", 1.0),
        WeightedPattern::new(r"(?i)\btrigonometr[ií]a\b", 1.0),
        WeightedPattern::new(r"(?i)\bc[aá]lculo\b", 1.0),
        WeightedPattern::new(r"(?i)\bderivada(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bintegral(es)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bestad[ií]stica(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bprobabilidad(es)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bporcentaje(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bfracci[oó]n(es)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bra[ií]z\s+cuadrada\b", 1.0),
        WeightedPattern::new(r"(?i)\blogaritmo(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bexponente(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bn[uú]mero(s)?\s+primo(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bfactorial\b", 1.0),
        
        // Operaciones básicas
        WeightedPattern::new(r"(?i)\bsuma\s+de\b", 1.0),
        WeightedPattern::new(r"(?i)\bsuma(r|ndo)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bresta\s+de\b", 1.0),
        WeightedPattern::new(r"(?i)\bresta(r|ndo)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bmultiplica(r|ci[oó]n)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bdivid(e|ir|iendo)\b", 1.0),
        WeightedPattern::new(r"(?i)\bdivisi[oó]n\b", 1.0),
        WeightedPattern::new(r"(?i)\bpromedio\b", 1.0),
        WeightedPattern::new(r"(?i)\bmedia\s+de\b", 1.0),
        WeightedPattern::new(r"(?i)\bmediana\b", 1.0),
        WeightedPattern::new(r"(?i)\bdesviaci[oó]n\s+est[aá]ndar\b", 1.0),
        
        // Word problems en español
        WeightedPattern::new(r"(?i)\bsi\s+tengo\s+\d+", 1.0),
        WeightedPattern::new(r"(?i)\bsi\s+hay\s+\d+", 1.0),
        WeightedPattern::new(r"(?i)\bcu[aá]ntos?\s+quedan\b", 1.0),
        WeightedPattern::new(r"(?i)\bcu[aá]ntos?\s+(hay|tiene|tengo)\b", 1.0),
        WeightedPattern::new(r"(?i)\ben\s+total\b", 1.0),
        WeightedPattern::new(r"(?i)\btotal\s+de\b", 1.0),
        WeightedPattern::new(r"(?i)\bprecio\s+total\b", 1.0),
        WeightedPattern::new(r"(?i)\bcosto\s+total\b", 1.0),
        WeightedPattern::new(r"(?i)\bdistancia\s+(de|a|entre)\b", 1.0),
        WeightedPattern::new(r"(?i)\bvelocidad\s+de\b", 1.0),
        WeightedPattern::new(r"(?i)\btiempo\s+(para|en)\b", 1.0),
        
        // Frases comunes
        WeightedPattern::new(r"(?i)\bcu[aá]l\s+es\s+el\s+resultado\b", 1.0),
        WeightedPattern::new(r"(?i)\bel\s+\d+\s*%\s+de\b", 1.0),
        WeightedPattern::new(r"(?i)\bconvertir\s+\d+", 1.0),
        WeightedPattern::new(r"(?i)\bm[aá]s\b.*\bmenos\b", 0.8),
        WeightedPattern::new(r"(?i)\bpor\b.*\bentre\b", 0.8),
    ]
}

/// Build Spanish code patterns
pub fn build_code_patterns_es() -> Vec<WeightedPattern> {
    vec![
        // Términos de programación
        WeightedPattern::new(r"(?i)\bc[oó]digo\b", 1.0),
        WeightedPattern::new(r"(?i)\bprograma(ci[oó]n|r)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bfunci[oó]n(es)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bclase(s)?\b", 0.8),
        WeightedPattern::new(r"(?i)\bm[eé]todo(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bvariable(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bbucle(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\barreglo(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\blista(s)?\b", 0.8),
        WeightedPattern::new(r"(?i)\bdiccionario(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bcadena(s)?\s+de\s+texto\b", 1.0),
        WeightedPattern::new(r"(?i)\bentero(s)?\b", 0.6),
        WeightedPattern::new(r"(?i)\bbooleano(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bnulo\b", 1.0),
        
        // Acciones de programación
        WeightedPattern::new(r"(?i)\bdepurar\b", 1.0),
        WeightedPattern::new(r"(?i)\bcompilar\b", 1.0),
        WeightedPattern::new(r"(?i)\bejecutar\b", 1.0),
        WeightedPattern::new(r"(?i)\bimplementar\b", 1.0),
        WeightedPattern::new(r"(?i)\brefactorizar\b", 1.0),
        WeightedPattern::new(r"(?i)\boptimizar\b", 1.0),
        WeightedPattern::new(r"(?i)\bcorregir\s+(el\s+)?(error|bug|fallo)\b", 1.2),
        WeightedPattern::new(r"(?i)\bescrib(e|ir)\s+(un(a)?\s+)?(c[oó]digo|funci[oó]n|programa)\b", 1.2),
        WeightedPattern::new(r"(?i)\bc[oó]mo\s+(hago|hacer|programo|codifico)\b", 1.0),
        
        // Términos técnicos
        WeightedPattern::new(r"(?i)\bsintaxis\b", 1.0),
        WeightedPattern::new(r"(?i)\bbiblioteca(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bpaquete(s)?\b", 0.8),
        WeightedPattern::new(r"(?i)\bm[oó]dulo(s)?\b", 0.8),
        WeightedPattern::new(r"(?i)\bimportar\b", 1.0),
        WeightedPattern::new(r"(?i)\bexportar\b", 0.8),
        WeightedPattern::new(r"(?i)\balgor[ií]tmo(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bestructura\s+de\s+datos\b", 1.2),
        WeightedPattern::new(r"(?i)\bbase\s+de\s+datos\b", 1.0),
        WeightedPattern::new(r"(?i)\bconsulta\s+(sql|de\s+base)\b", 1.2),
        WeightedPattern::new(r"(?i)\bexpresi[oó]n\s+regular\b", 1.2),
    ]
}

/// Build Spanish reasoning patterns
pub fn build_reasoning_patterns_es() -> Vec<WeightedPattern> {
    vec![
        // Análisis - alta prioridad
        WeightedPattern::new(r"(?i)\banaliz(a|ar|o)\b", 1.0),
        WeightedPattern::new(r"(?i)\ban[aá]lisis\b", 1.0),
        WeightedPattern::new(r"(?i)\bcompar(a|ar|o)\b", 1.0),
        WeightedPattern::new(r"(?i)\bcomparaci[oó]n\b", 1.0),
        WeightedPattern::new(r"(?i)\bcontras?t(a|ar|o)\b", 1.0),
        WeightedPattern::new(r"(?i)\beval[uú](a|ar|o)\b", 1.0),
        WeightedPattern::new(r"(?i)\bevaluaci[oó]n\b", 1.0),
        WeightedPattern::new(r"(?i)\bcritica(r)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bcr[ií]tico\b", 1.0),
        
        // Pros y contras - muy alta prioridad
        WeightedPattern::new(r"(?i)\bventajas?\s+y\s+desventajas?\b", 2.0),
        WeightedPattern::new(r"(?i)\bpros?\s+y\s+contras?\b", 2.0),
        WeightedPattern::new(r"(?i)\bventajas?\b", 1.0),
        WeightedPattern::new(r"(?i)\bdesventajas?\b", 1.0),
        WeightedPattern::new(r"(?i)\bbeneficios?\b", 1.0),
        WeightedPattern::new(r"(?i)\binconvenientes?\b", 1.0),
        
        // Por qué - alta prioridad
        WeightedPattern::new(r"(?i)^por\s+qu[eé]\b", 2.0),
        WeightedPattern::new(r"(?i)\bpor\s+qu[eé]\s+(es|son|est[aá]|funciona)\b", 1.5),
        WeightedPattern::new(r"(?i)\bexplica(r)?\s+por\s+qu[eé]\b", 1.5),
        WeightedPattern::new(r"(?i)\braz[oó]n(es|amiento)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bl[oó]gica?\b", 1.0),
        WeightedPattern::new(r"(?i)\bargumento(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bhip[oó]tesis\b", 1.0),
        WeightedPattern::new(r"(?i)\bconclusi[oó]n(es)?\b", 1.0),
        WeightedPattern::new(r"(?i)\binferencia(s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bdeducci[oó]n\b", 1.0),
        WeightedPattern::new(r"(?i)\binducci[oó]n\b", 1.0),
        
        // Pensar y decidir
        WeightedPattern::new(r"(?i)\bpiens(a|o)\s+(en|sobre)\b", 1.0),
        WeightedPattern::new(r"(?i)\bconsidera(r)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bsopes(a|ar)\b", 1.0),
        WeightedPattern::new(r"(?i)\bjuzga(r)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bdecid(e|ir|o)\b", 1.0),
        WeightedPattern::new(r"(?i)\bdecisi[oó]n\b", 1.0),
        
        // Preguntas hipotéticas - alta prioridad
        WeightedPattern::new(r"(?i)\bqu[eé]\s+pasar[ií]a\s+si\b", 2.0),
        WeightedPattern::new(r"(?i)\bimagina(r)?\s+que\b", 1.5),
        WeightedPattern::new(r"(?i)\bsupon(er|gamos|iendo)\b", 1.5),
        WeightedPattern::new(r"(?i)\bhipot[eé]ticamente\b", 1.5),
        WeightedPattern::new(r"(?i)\ben\s+teor[ií]a\b", 1.0),
        WeightedPattern::new(r"(?i)\bqu[eé]\s+opinas?\b", 1.0),
        WeightedPattern::new(r"(?i)\bdeber[ií]a\b", 1.0),
        WeightedPattern::new(r"(?i)^deber[ií]a\s+(yo|usar|aprender|elegir)\b", 2.0),
    ]
}

/// Build Spanish tools patterns
pub fn build_tools_patterns_es() -> Vec<WeightedPattern> {
    vec![
        // Búsqueda - alta prioridad
        WeightedPattern::new(r"(?i)\bbusca(r)?\s+(en\s+)?(la\s+)?web\b", 1.5),
        WeightedPattern::new(r"(?i)\bbusca(r)?\s+(en\s+)?internet\b", 1.5),
        WeightedPattern::new(r"(?i)\bbusca(r)?\s+informaci[oó]n\b", 1.0),
        WeightedPattern::new(r"(?i)\bencontrar\s+(informaci[oó]n|datos|resultados)\b", 1.0),
        WeightedPattern::new(r"(?i)\bgooglea(r)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bnavega(r)?\b", 0.8),
        WeightedPattern::new(r"(?i)\babri(r)?\s+(un(a)?\s+)?(archivo|url|enlace|p[aá]gina)\b", 1.0),
        
        // Archivos
        WeightedPattern::new(r"(?i)\bdescargar\b", 1.0),
        WeightedPattern::new(r"(?i)\bsubir\b", 0.8),
        WeightedPattern::new(r"(?i)\bguardar\s+(en|como)\b", 1.0),
        WeightedPattern::new(r"(?i)\bexportar\s+(a|como)\b", 1.0),
        WeightedPattern::new(r"(?i)\bconvertir\s+a\b", 1.0),
        
        // Generación de imágenes - alta prioridad
        WeightedPattern::new(r"(?i)\bgenera(r)?\s+(una?\s+)?(imagen|foto|dibujo|ilustraci[oó]n)\b", 1.5),
        WeightedPattern::new(r"(?i)\bcrea(r)?\s+(una?\s+)?(imagen|foto|dibujo|ilustraci[oó]n)\b", 1.5),
        WeightedPattern::new(r"(?i)\bdibuja(r)?\s+(un(a)?|me)\b", 1.5),
        WeightedPattern::new(r"(?i)\bhaz(me)?\s+(una?\s+)?(imagen|foto|dibujo)\b", 1.5),
        
        // Crear documentos
        WeightedPattern::new(r"(?i)\bcrea(r)?\s+(un(a)?\s+)?(archivo|documento|informe)\b", 1.0),
        WeightedPattern::new(r"(?i)\benvia(r)?\s+(un(a)?\s+)?(correo|email|mensaje)\b", 1.0),
        WeightedPattern::new(r"(?i)\bprograma(r)?\s+(una?\s+)?(reuni[oó]n|cita)\b", 1.0),
        WeightedPattern::new(r"(?i)\brecordatorio\b", 1.0),
        WeightedPattern::new(r"(?i)\balarma\b", 1.0),
        WeightedPattern::new(r"(?i)\btemporizador\b", 1.0),
        WeightedPattern::new(r"(?i)\bcalendario\b", 1.0),
        
        // Información en tiempo real
        WeightedPattern::new(r"(?i)\bclima\b", 1.0),
        WeightedPattern::new(r"(?i)\btiempo\s+(que\s+)?hace\b", 1.0),
        WeightedPattern::new(r"(?i)\btemperatura\b", 1.0),
        WeightedPattern::new(r"(?i)\bnoticias\b", 1.0),
        WeightedPattern::new(r"(?i)\bprecio\s+(de\s+)?(las?\s+)?acciones?\b", 1.0),
        WeightedPattern::new(r"(?i)\bcotizaci[oó]n\b", 1.0),
        WeightedPattern::new(r"(?i)\btraduc(e|ir)\b", 1.0),
        WeightedPattern::new(r"(?i)\btraducci[oó]n\b", 1.0),
        
        // Últimas noticias
        WeightedPattern::new(r"(?i)\b[uú]ltimas?\s+noticias?\b", 1.2),
        WeightedPattern::new(r"(?i)\bqu[eé]\s+hay\s+de\s+nuevo\b", 1.0),
    ]
}

/// Build Spanish greeting patterns
pub fn build_greeting_patterns_es() -> Vec<WeightedPattern> {
    vec![
        // Saludos directos - muy alta prioridad
        WeightedPattern::new(r"(?i)^hola\b", 2.0),
        WeightedPattern::new(r"(?i)^buenos?\s+d[ií]as?\b", 2.0),
        WeightedPattern::new(r"(?i)^buenas?\s+tardes?\b", 2.0),
        WeightedPattern::new(r"(?i)^buenas?\s+noches?\b", 2.0),
        WeightedPattern::new(r"(?i)^qu[eé]\s+tal\b", 2.0),
        WeightedPattern::new(r"(?i)^qu[eé]\s+onda\b", 2.0),
        WeightedPattern::new(r"(?i)^qu[eé]\s+hay\b", 1.5),
        WeightedPattern::new(r"(?i)^saludos?\b", 2.0),
        WeightedPattern::new(r"(?i)^hey\b", 1.5),
        
        // Cómo estás
        WeightedPattern::new(r"(?i)^c[oó]mo\s+est[aá]s?\b", 2.0),
        WeightedPattern::new(r"(?i)^c[oó]mo\s+te\s+va\b", 2.0),
        WeightedPattern::new(r"(?i)^c[oó]mo\s+andas?\b", 1.5),
        WeightedPattern::new(r"(?i)\bmucho\s+gusto\b", 1.5),
        WeightedPattern::new(r"(?i)\bencantado\s+de\s+conocerte\b", 1.5),
        WeightedPattern::new(r"(?i)\bun\s+placer\b", 1.5),
        
        // Despedidas
        WeightedPattern::new(r"(?i)\badi[oó]s\b", 1.0),
        WeightedPattern::new(r"(?i)\bhasta\s+(luego|pronto|ma[nñ]ana|la\s+vista)\b", 1.0),
        WeightedPattern::new(r"(?i)\bchao\b", 1.0),
        WeightedPattern::new(r"(?i)\bchau\b", 1.0),
        WeightedPattern::new(r"(?i)\bnos\s+vemos\b", 1.0),
        WeightedPattern::new(r"(?i)\bcu[ií]date\b", 1.0),
        WeightedPattern::new(r"(?i)\bque\s+te\s+vaya\s+bien\b", 1.0),
        
        // Cortesía
        WeightedPattern::new(r"(?i)^gracias\b", 1.0),
        WeightedPattern::new(r"(?i)^muchas\s+gracias\b", 1.0),
        WeightedPattern::new(r"(?i)^por\s+favor\b", 1.0),
        WeightedPattern::new(r"(?i)^perd[oó]n\b", 1.0),
        WeightedPattern::new(r"(?i)^disculpa\b", 1.0),
        WeightedPattern::new(r"(?i)^lo\s+siento\b", 1.0),
        
        // Sobre el asistente
        WeightedPattern::new(r"(?i)\bqui[eé]n\s+eres\b", 1.5),
        WeightedPattern::new(r"(?i)\bc[oó]mo\s+te\s+llamas\b", 1.5),
        WeightedPattern::new(r"(?i)\bcu[aá]l\s+es\s+tu\s+nombre\b", 1.5),
        WeightedPattern::new(r"(?i)\bqu[eé]\s+puedes\s+hacer\b", 1.5),
        WeightedPattern::new(r"(?i)\bqu[eé]\s+sabes\s+hacer\b", 1.5),
        WeightedPattern::new(r"(?i)\bcu[eé]ntame\s+(de|sobre)\s+ti\b", 1.5),
    ]
}

/// Build Spanish factual patterns
pub fn build_factual_patterns_es() -> Vec<WeightedPattern> {
    vec![
        // Preguntas qué/quién/cuándo/dónde - alta prioridad
        WeightedPattern::new(r"(?i)^qu[eé]\s+es\b", 1.5),
        WeightedPattern::new(r"(?i)^qu[eé]\s+son\b", 1.5),
        WeightedPattern::new(r"(?i)^qu[eé]\s+fue\b", 1.5),
        WeightedPattern::new(r"(?i)^qu[eé]\s+eran?\b", 1.5),
        WeightedPattern::new(r"(?i)^qui[eé]n\s+(es|fue|era)\b", 1.5),
        WeightedPattern::new(r"(?i)^qui[eé]nes\s+(son|fueron|eran)\b", 1.5),
        WeightedPattern::new(r"(?i)^cu[aá]ndo\s+(es|fue|era|ser[aá])\b", 1.5),
        WeightedPattern::new(r"(?i)^d[oó]nde\s+(es|est[aá]|queda|se\s+encuentra)\b", 1.5),
        WeightedPattern::new(r"(?i)^cu[aá]l\s+(es|fue|era)\b", 1.5),
        WeightedPattern::new(r"(?i)^cu[aá]ntos?\s+(hay|tiene|son|eran)\b", 1.2),
        
        // Definiciones
        WeightedPattern::new(r"(?i)\bdefin(e|ir|ici[oó]n)\b", 1.0),
        WeightedPattern::new(r"(?i)\bsignificado\s+de\b", 1.0),
        WeightedPattern::new(r"(?i)\bqu[eé]\s+significa\b", 1.0),
        
        // Historia y origen
        WeightedPattern::new(r"(?i)\bhistoria\s+de\b", 1.0),
        WeightedPattern::new(r"(?i)\borigen\s+de\b", 1.0),
        WeightedPattern::new(r"(?i)\bdatos?\s+(sobre|de|acerca)\b", 1.0),
        WeightedPattern::new(r"(?i)\binformaci[oó]n\s+(sobre|de|acerca)\b", 1.0),
        
        // Cuéntame/explica
        WeightedPattern::new(r"(?i)\bcu[eé]ntame\s+(sobre|de|acerca)\b", 1.0),
        WeightedPattern::new(r"(?i)\bh[aá]blame\s+(sobre|de|acerca)\b", 1.0),
        WeightedPattern::new(r"(?i)\bexplica(r)?\s+(qu[eé]|c[oó]mo|el|la|los|las)\b", 1.0),
        WeightedPattern::new(r"(?i)\bdescrib(e|ir)\b", 1.0),
        
        // Datos específicos
        WeightedPattern::new(r"(?i)\bcapital\s+de\b", 1.2),
        WeightedPattern::new(r"(?i)\bpoblaci[oó]n\s+de\b", 1.2),
        WeightedPattern::new(r"(?i)\bpresidente\s+de\b", 1.2),
        WeightedPattern::new(r"(?i)\bdirector\s+de\b", 1.0),
        WeightedPattern::new(r"(?i)\bfundador\s+de\b", 1.0),
        
        // Inventores y descubridores - alta prioridad
        WeightedPattern::new(r"(?i)\binventor\s+de\b", 1.5),
        WeightedPattern::new(r"(?i)\bquien\s+invent[oó]\b", 2.0),
        WeightedPattern::new(r"(?i)\bquien\s+descubri[oó]\b", 2.0),
        WeightedPattern::new(r"(?i)\binvent[oó]\b", 1.5),
        WeightedPattern::new(r"(?i)\bdescubri[oó]\b", 1.5),
        WeightedPattern::new(r"(?i)\bdescubridor\s+de\b", 1.5),
        WeightedPattern::new(r"(?i)\bquien\s+cre[oó]\b", 1.5),
        WeightedPattern::new(r"(?i)\bcreador\s+de\b", 1.5),
        WeightedPattern::new(r"(?i)\bautor\s+de\b", 1.0),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    fn test_patterns_match(patterns: &[WeightedPattern], text: &str) -> bool {
        for p in patterns {
            if let Ok(re) = Regex::new(p.pattern) {
                if re.is_match(text) {
                    return true;
                }
            }
        }
        false
    }

    #[test]
    fn test_math_spanish() {
        let patterns = build_math_patterns_es();
        assert!(test_patterns_match(&patterns, "cuánto es 5 + 3"));
        assert!(test_patterns_match(&patterns, "calcula la raíz cuadrada"));
        assert!(test_patterns_match(&patterns, "resuelve la ecuación"));
        assert!(test_patterns_match(&patterns, "si tengo 5 manzanas"));
    }

    #[test]
    fn test_code_spanish() {
        let patterns = build_code_patterns_es();
        assert!(test_patterns_match(&patterns, "escribe una función"));
        assert!(test_patterns_match(&patterns, "cómo implementar"));
        assert!(test_patterns_match(&patterns, "depurar el código"));
    }

    #[test]
    fn test_greeting_spanish() {
        let patterns = build_greeting_patterns_es();
        assert!(test_patterns_match(&patterns, "hola"));
        assert!(test_patterns_match(&patterns, "buenos días"));
        assert!(test_patterns_match(&patterns, "cómo estás"));
        assert!(test_patterns_match(&patterns, "quién eres"));
    }

    #[test]
    fn test_factual_spanish() {
        let patterns = build_factual_patterns_es();
        assert!(test_patterns_match(&patterns, "qué es la fotosíntesis"));
        assert!(test_patterns_match(&patterns, "quién inventó el teléfono"));
        assert!(test_patterns_match(&patterns, "capital de Francia"));
    }

    #[test]
    fn test_reasoning_spanish() {
        let patterns = build_reasoning_patterns_es();
        assert!(test_patterns_match(&patterns, "ventajas y desventajas"));
        assert!(test_patterns_match(&patterns, "por qué funciona"));
        assert!(test_patterns_match(&patterns, "qué pasaría si"));
    }

    #[test]
    fn test_tools_spanish() {
        let patterns = build_tools_patterns_es();
        assert!(test_patterns_match(&patterns, "buscar en la web"));
        assert!(test_patterns_match(&patterns, "generar una imagen"));
        assert!(test_patterns_match(&patterns, "traducir al inglés"));
    }
}
