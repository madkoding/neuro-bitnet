#!/usr/bin/env python3
"""
BitNet Stress Test
==================
Pruebas de carga y concurrencia para neuro-bitnet.

Incluye:
- Pruebas de latencia
- Pruebas de throughput
- Pruebas de concurrencia
- Pruebas de contexto largo

Uso:
    python stress_test.py [--url URL] [--requests N] [--concurrent N]
"""

import json
import time
import argparse
import statistics
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from typing import Optional

try:
    import requests
except ImportError:
    print("❌ Falta el módulo 'requests'. Instálalo con: pip install requests")
    exit(1)

# =============================================================================
# Configuración
# =============================================================================

DEFAULT_URL = "http://localhost:11435"
TIMEOUT = 180

@dataclass
class RequestResult:
    success: bool
    response_time: float
    tokens: int
    error: Optional[str] = None

# =============================================================================
# Cliente
# =============================================================================

class BitNetClient:
    def __init__(self, base_url: str):
        self.base_url = base_url.rstrip('/')
    
    def health_check(self) -> bool:
        try:
            r = requests.get(f"{self.base_url}/health", timeout=5)
            return r.status_code == 200
        except:
            return False
    
    def chat(self, prompt: str, max_tokens: int = 50) -> RequestResult:
        start = time.time()
        try:
            r = requests.post(
                f"{self.base_url}/v1/chat/completions",
                json={
                    "model": "bitnet",
                    "messages": [{"role": "user", "content": prompt}],
                    "max_tokens": max_tokens,
                    "temperature": 0.3
                },
                timeout=TIMEOUT
            )
            elapsed = time.time() - start
            
            if r.status_code != 200:
                return RequestResult(False, elapsed, 0, f"HTTP {r.status_code}")
            
            data = r.json()
            tokens = data.get("usage", {}).get("completion_tokens", 0)
            
            return RequestResult(True, elapsed, tokens)
        except Exception as e:
            return RequestResult(False, time.time() - start, 0, str(e))

# =============================================================================
# Tests de Estrés
# =============================================================================

def test_latency(client: BitNetClient, num_requests: int = 10) -> dict:
    """Prueba de latencia con requests secuenciales."""
    print(f"\n📊 Test de Latencia ({num_requests} requests secuenciales)")
    print("-" * 60)
    
    results = []
    prompts = [
        "¡Hola!",
        "¿Cuánto es 2+2?",
        "Di hola",
        "Cuenta hasta 3",
        "Nombra un color"
    ]
    
    for i in range(num_requests):
        prompt = prompts[i % len(prompts)]
        print(f"  [{i+1}/{num_requests}] Enviando...", end=" ", flush=True)
        result = client.chat(prompt, max_tokens=30)
        results.append(result)
        
        if result.success:
            print(f"✅ {result.response_time:.2f}s ({result.tokens} tokens)")
        else:
            print(f"❌ {result.error}")
    
    successful = [r for r in results if r.success]
    
    if successful:
        times = [r.response_time for r in successful]
        return {
            "total_requests": num_requests,
            "successful": len(successful),
            "failed": num_requests - len(successful),
            "min_latency": min(times),
            "max_latency": max(times),
            "avg_latency": statistics.mean(times),
            "median_latency": statistics.median(times),
            "std_dev": statistics.stdev(times) if len(times) > 1 else 0
        }
    
    return {"total_requests": num_requests, "successful": 0, "failed": num_requests}

def test_throughput(client: BitNetClient, duration: int = 30) -> dict:
    """Prueba de throughput: cuántos requests en un tiempo dado."""
    print(f"\n📊 Test de Throughput ({duration} segundos)")
    print("-" * 60)
    
    results = []
    start = time.time()
    count = 0
    
    while time.time() - start < duration:
        count += 1
        print(f"  Request #{count}...", end=" ", flush=True)
        result = client.chat("Hola", max_tokens=20)
        results.append(result)
        
        if result.success:
            print(f"✅ {result.response_time:.2f}s")
        else:
            print(f"❌ {result.error}")
    
    elapsed = time.time() - start
    successful = [r for r in results if r.success]
    total_tokens = sum(r.tokens for r in successful)
    
    return {
        "duration": elapsed,
        "total_requests": len(results),
        "successful": len(successful),
        "failed": len(results) - len(successful),
        "requests_per_second": len(results) / elapsed,
        "successful_per_second": len(successful) / elapsed,
        "total_tokens": total_tokens,
        "tokens_per_second": total_tokens / elapsed
    }

def test_concurrency(client: BitNetClient, num_requests: int = 10, max_workers: int = 4) -> dict:
    """Prueba de concurrencia con requests paralelos."""
    print(f"\n📊 Test de Concurrencia ({num_requests} requests, {max_workers} workers)")
    print("-" * 60)
    
    prompts = [f"Cuenta del 1 al {i+3}" for i in range(num_requests)]
    results = []
    
    start = time.time()
    
    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        futures = {executor.submit(client.chat, p, 50): p for p in prompts}
        
        for i, future in enumerate(as_completed(futures), 1):
            result = future.result()
            results.append(result)
            
            if result.success:
                print(f"  [{i}/{num_requests}] ✅ {result.response_time:.2f}s ({result.tokens} tokens)")
            else:
                print(f"  [{i}/{num_requests}] ❌ {result.error}")
    
    elapsed = time.time() - start
    successful = [r for r in results if r.success]
    
    if successful:
        times = [r.response_time for r in successful]
        return {
            "total_time": elapsed,
            "total_requests": num_requests,
            "concurrent_workers": max_workers,
            "successful": len(successful),
            "failed": num_requests - len(successful),
            "avg_latency": statistics.mean(times),
            "requests_per_second": num_requests / elapsed
        }
    
    return {"total_requests": num_requests, "successful": 0, "failed": num_requests}

def test_context_length(client: BitNetClient) -> dict:
    """Prueba con diferentes tamaños de contexto."""
    print(f"\n📊 Test de Contexto Largo")
    print("-" * 60)
    
    results = {}
    
    # Diferentes tamaños de prompt
    sizes = [100, 500, 1000, 2000]
    
    for size in sizes:
        # Generar prompt de aproximadamente 'size' caracteres
        prompt = "Cuéntame sobre " + ("la historia de la computación " * (size // 30))[:size]
        
        print(f"  Contexto ~{size} chars...", end=" ", flush=True)
        result = client.chat(prompt, max_tokens=100)
        
        if result.success:
            print(f"✅ {result.response_time:.2f}s ({result.tokens} tokens)")
            results[size] = {
                "success": True,
                "time": result.response_time,
                "tokens": result.tokens
            }
        else:
            print(f"❌ {result.error}")
            results[size] = {"success": False, "error": result.error}
    
    return results

def test_token_generation(client: BitNetClient) -> dict:
    """Prueba de generación con diferentes max_tokens."""
    print(f"\n📊 Test de Generación de Tokens")
    print("-" * 60)
    
    results = {}
    token_sizes = [10, 50, 100, 200]
    
    prompt = "Escribe una explicación detallada sobre la inteligencia artificial."
    
    for max_tokens in token_sizes:
        print(f"  Max tokens: {max_tokens}...", end=" ", flush=True)
        result = client.chat(prompt, max_tokens=max_tokens)
        
        if result.success:
            tps = result.tokens / result.response_time if result.response_time > 0 else 0
            print(f"✅ {result.response_time:.2f}s ({result.tokens} tokens, {tps:.1f} t/s)")
            results[max_tokens] = {
                "success": True,
                "time": result.response_time,
                "tokens_generated": result.tokens,
                "tokens_per_second": tps
            }
        else:
            print(f"❌ {result.error}")
            results[max_tokens] = {"success": False, "error": result.error}
    
    return results

# =============================================================================
# Reporte
# =============================================================================

def print_report(latency: dict, throughput: dict, concurrency: dict, context: dict, tokens: dict):
    """Imprime reporte final."""
    
    print("\n" + "=" * 70)
    print("📈 REPORTE DE ESTRÉS")
    print("=" * 70)
    
    print("\n🔵 LATENCIA")
    if latency.get("successful", 0) > 0:
        print(f"   Requests exitosos: {latency['successful']}/{latency['total_requests']}")
        print(f"   Latencia mínima:   {latency['min_latency']:.3f}s")
        print(f"   Latencia máxima:   {latency['max_latency']:.3f}s")
        print(f"   Latencia promedio: {latency['avg_latency']:.3f}s")
        print(f"   Latencia mediana:  {latency['median_latency']:.3f}s")
        print(f"   Desviación std:    {latency['std_dev']:.3f}s")
    else:
        print("   ❌ No hay datos exitosos")
    
    print("\n🔵 THROUGHPUT")
    if throughput.get("successful", 0) > 0:
        print(f"   Duración:              {throughput['duration']:.1f}s")
        print(f"   Requests totales:      {throughput['total_requests']}")
        print(f"   Requests exitosos:     {throughput['successful']}")
        print(f"   Requests/segundo:      {throughput['requests_per_second']:.2f}")
        print(f"   Tokens totales:        {throughput['total_tokens']}")
        print(f"   Tokens/segundo:        {throughput['tokens_per_second']:.1f}")
    else:
        print("   ❌ No hay datos exitosos")
    
    print("\n🔵 CONCURRENCIA")
    if concurrency.get("successful", 0) > 0:
        print(f"   Workers:              {concurrency['concurrent_workers']}")
        print(f"   Tiempo total:         {concurrency['total_time']:.2f}s")
        print(f"   Requests exitosos:    {concurrency['successful']}/{concurrency['total_requests']}")
        print(f"   Latencia promedio:    {concurrency['avg_latency']:.3f}s")
        print(f"   Requests/segundo:     {concurrency['requests_per_second']:.2f}")
    else:
        print("   ❌ No hay datos exitosos")
    
    print("\n🔵 CONTEXTO LARGO")
    for size, result in context.items():
        if result.get("success"):
            print(f"   {size:>5} chars: {result['time']:.2f}s ({result['tokens']} tokens)")
        else:
            print(f"   {size:>5} chars: ❌ {result.get('error', 'Failed')}")
    
    print("\n🔵 GENERACIÓN DE TOKENS")
    for max_tok, result in tokens.items():
        if result.get("success"):
            print(f"   max={max_tok:>3}: {result['tokens_generated']:>3} tokens en {result['time']:.2f}s ({result['tokens_per_second']:.1f} t/s)")
        else:
            print(f"   max={max_tok:>3}: ❌ {result.get('error', 'Failed')}")
    
    print("\n" + "=" * 70)

# =============================================================================
# Main
# =============================================================================

def main():
    parser = argparse.ArgumentParser(description="BitNet Stress Test")
    parser.add_argument("--url", default=DEFAULT_URL, help=f"URL del servidor (default: {DEFAULT_URL})")
    parser.add_argument("--requests", type=int, default=10, help="Número de requests para tests (default: 10)")
    parser.add_argument("--concurrent", type=int, default=4, help="Workers concurrentes (default: 4)")
    parser.add_argument("--duration", type=int, default=30, help="Duración del test de throughput en segundos (default: 30)")
    args = parser.parse_args()
    
    client = BitNetClient(args.url)
    
    print(f"\n🔗 Conectando a {args.url}...")
    
    if not client.health_check():
        print(f"❌ Error: No se puede conectar a {args.url}")
        exit(1)
    
    print("✅ Servidor disponible")
    
    # Ejecutar tests
    latency = test_latency(client, args.requests)
    throughput = test_throughput(client, args.duration)
    concurrency = test_concurrency(client, args.requests, args.concurrent)
    context = test_context_length(client)
    tokens = test_token_generation(client)
    
    # Reporte
    print_report(latency, throughput, concurrency, context, tokens)

if __name__ == "__main__":
    main()
