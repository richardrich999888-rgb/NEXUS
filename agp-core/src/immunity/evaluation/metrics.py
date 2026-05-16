"""
Evaluation Metrics for AIS-ASI.

Provides comprehensive metrics for immune system performance:
- Confusion matrix (TP, FP, TN, FN)
- Precision, Recall, F1
- ROC/AUC analysis
- Response time distribution
- Memory hit rate
- Antibody diversity
"""

import torch
import numpy as np
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass, field
import time
import json


@dataclass
class ConfusionMatrix:
    """Confusion matrix for threat detection."""
    true_positives: int = 0
    false_positives: int = 0
    true_negatives: int = 0
    false_negatives: int = 0
    
    def update(self, predicted: bool, actual: bool):
        """Update matrix with a single prediction."""
        if predicted and actual:
            self.true_positives += 1
        elif predicted and not actual:
            self.false_positives += 1
        elif not predicted and not actual:
            self.true_negatives += 1
        else:
            self.false_negatives += 1
    
    @property
    def total(self) -> int:
        return self.true_positives + self.false_positives + self.true_negatives + self.false_negatives
    
    @property
    def accuracy(self) -> float:
        if self.total == 0:
            return 0.0
        return (self.true_positives + self.true_negatives) / self.total
    
    @property
    def precision(self) -> float:
        denom = self.true_positives + self.false_positives
        return self.true_positives / denom if denom > 0 else 0.0
    
    @property
    def recall(self) -> float:
        denom = self.true_positives + self.false_negatives
        return self.true_positives / denom if denom > 0 else 0.0
    
    @property
    def f1(self) -> float:
        p, r = self.precision, self.recall
        return 2 * p * r / (p + r) if (p + r) > 0 else 0.0
    
    @property
    def specificity(self) -> float:
        denom = self.true_negatives + self.false_positives
        return self.true_negatives / denom if denom > 0 else 0.0
    
    @property
    def false_positive_rate(self) -> float:
        return 1.0 - self.specificity
    
    @property
    def false_negative_rate(self) -> float:
        return 1.0 - self.recall
    
    def to_dict(self) -> Dict:
        return {
            'tp': self.true_positives,
            'fp': self.false_positives,
            'tn': self.true_negatives,
            'fn': self.false_negatives,
            'accuracy': self.accuracy,
            'precision': self.precision,
            'recall': self.recall,
            'f1': self.f1,
            'specificity': self.specificity,
            'fpr': self.false_positive_rate,
            'fnr': self.false_negative_rate
        }
    
    def __str__(self) -> str:
        return (f"ConfusionMatrix(TP={self.true_positives}, FP={self.false_positives}, "
                f"TN={self.true_negatives}, FN={self.false_negatives}, "
                f"Acc={self.accuracy:.3f}, F1={self.f1:.3f})")


@dataclass
class ROCAnalysis:
    """ROC curve analysis for threshold tuning."""
    thresholds: List[float] = field(default_factory=list)
    tpr_values: List[float] = field(default_factory=list)
    fpr_values: List[float] = field(default_factory=list)
    
    def compute_from_scores(
        self,
        scores: List[float],
        labels: List[bool],
        num_thresholds: int = 100
    ):
        """Compute ROC curve from prediction scores and labels."""
        self.thresholds = np.linspace(0, 1, num_thresholds).tolist()
        self.tpr_values = []
        self.fpr_values = []
        
        for thresh in self.thresholds:
            cm = ConfusionMatrix()
            for score, label in zip(scores, labels):
                predicted = score >= thresh
                cm.update(predicted, label)
            
            self.tpr_values.append(cm.recall)
            self.fpr_values.append(cm.false_positive_rate)
    
    @property
    def auc(self) -> float:
        """Compute Area Under ROC Curve using trapezoidal rule."""
        if len(self.fpr_values) < 2:
            return 0.0
        
        # Sort by FPR for integration
        points = sorted(zip(self.fpr_values, self.tpr_values))
        fpr_sorted = [p[0] for p in points]
        tpr_sorted = [p[1] for p in points]
        
        # Trapezoidal integration
        auc = 0.0
        for i in range(1, len(fpr_sorted)):
            auc += (fpr_sorted[i] - fpr_sorted[i-1]) * (tpr_sorted[i] + tpr_sorted[i-1]) / 2
        
        return auc
    
    def optimal_threshold(self, target_fpr: float = 0.05) -> float:
        """Find threshold that achieves target FPR."""
        for thresh, fpr in zip(self.thresholds, self.fpr_values):
            if fpr <= target_fpr:
                return thresh
        return 0.5


class ImmuneMetrics:
    """
    Comprehensive metrics for immune system evaluation.
    
    Tracks:
    - Detection performance (confusion matrix)
    - Response times
    - Memory efficiency
    - Antibody evolution
    - System health
    """
    
    def __init__(self):
        self.confusion_matrix = ConfusionMatrix()
        self.response_times: List[float] = []
        self.memory_hits: int = 0
        self.memory_misses: int = 0
        self.antibody_diversity_history: List[float] = []
        self.threat_severity_distribution: Dict[str, List[float]] = {}
        self.detection_by_type: Dict[str, ConfusionMatrix] = {}
        
        # Per-epoch metrics
        self.epoch_metrics: List[Dict] = []
        
    def record_prediction(
        self,
        predicted: bool,
        actual: bool,
        threat_type: str = "unknown",
        response_time_ms: float = 0.0,
        memory_hit: bool = False,
        severity: float = 0.0
    ):
        """Record a single prediction."""
        # Update main confusion matrix
        self.confusion_matrix.update(predicted, actual)
        
        # Response time
        self.response_times.append(response_time_ms)
        
        # Memory tracking
        if memory_hit:
            self.memory_hits += 1
        elif actual:  # Should have been detected
            self.memory_misses += 1
        
        # Per-type tracking
        if threat_type not in self.detection_by_type:
            self.detection_by_type[threat_type] = ConfusionMatrix()
        self.detection_by_type[threat_type].update(predicted, actual)
        
        # Severity distribution
        if actual:
            if threat_type not in self.threat_severity_distribution:
                self.threat_severity_distribution[threat_type] = []
            self.threat_severity_distribution[threat_type].append(severity)
    
    def record_antibody_diversity(self, diversity: float):
        """Record antibody pool diversity."""
        self.antibody_diversity_history.append(diversity)
    
    def end_epoch(self, epoch: int):
        """End epoch and record metrics."""
        metrics = {
            'epoch': epoch,
            **self.confusion_matrix.to_dict(),
            'avg_response_time_ms': np.mean(self.response_times) if self.response_times else 0,
            'p95_response_time_ms': np.percentile(self.response_times, 95) if self.response_times else 0,
            'memory_hit_rate': self.memory_hits / (self.memory_hits + self.memory_misses + 1e-8),
            'antibody_diversity': self.antibody_diversity_history[-1] if self.antibody_diversity_history else 0
        }
        self.epoch_metrics.append(metrics)
        return metrics
    
    def get_summary(self) -> Dict:
        """Get comprehensive metrics summary."""
        return {
            'overall': self.confusion_matrix.to_dict(),
            'response_time': {
                'mean_ms': np.mean(self.response_times) if self.response_times else 0,
                'std_ms': np.std(self.response_times) if self.response_times else 0,
                'p50_ms': np.percentile(self.response_times, 50) if self.response_times else 0,
                'p95_ms': np.percentile(self.response_times, 95) if self.response_times else 0,
                'p99_ms': np.percentile(self.response_times, 99) if self.response_times else 0,
            },
            'memory': {
                'hits': self.memory_hits,
                'misses': self.memory_misses,
                'hit_rate': self.memory_hits / (self.memory_hits + self.memory_misses + 1e-8)
            },
            'antibody_diversity': {
                'current': self.antibody_diversity_history[-1] if self.antibody_diversity_history else 0,
                'mean': np.mean(self.antibody_diversity_history) if self.antibody_diversity_history else 0,
                'trend': (self.antibody_diversity_history[-1] - self.antibody_diversity_history[0]) 
                         if len(self.antibody_diversity_history) > 1 else 0
            },
            'by_threat_type': {
                t_type: cm.to_dict()
                for t_type, cm in self.detection_by_type.items()
            },
            'epochs': self.epoch_metrics
        }
    
    def save(self, filepath: str):
        """Save metrics to JSON."""
        with open(filepath, 'w') as f:
            json.dump(self.get_summary(), f, indent=2)
        print(f"📊 Metrics saved to {filepath}")
    
    def print_summary(self):
        """Print formatted summary."""
        summary = self.get_summary()
        
        print("\n" + "="*60)
        print("📊 IMMUNE SYSTEM EVALUATION METRICS")
        print("="*60)
        
        print("\n📈 Overall Performance:")
        print(f"  Accuracy:  {summary['overall']['accuracy']:.4f}")
        print(f"  Precision: {summary['overall']['precision']:.4f}")
        print(f"  Recall:    {summary['overall']['recall']:.4f}")
        print(f"  F1 Score:  {summary['overall']['f1']:.4f}")
        print(f"  FPR:       {summary['overall']['fpr']:.4f}")
        print(f"  FNR:       {summary['overall']['fnr']:.4f}")
        
        print("\n⏱️ Response Time:")
        print(f"  Mean:  {summary['response_time']['mean_ms']:.2f}ms")
        print(f"  P50:   {summary['response_time']['p50_ms']:.2f}ms")
        print(f"  P95:   {summary['response_time']['p95_ms']:.2f}ms")
        print(f"  P99:   {summary['response_time']['p99_ms']:.2f}ms")
        
        print("\n🧠 Memory Performance:")
        print(f"  Hit Rate: {summary['memory']['hit_rate']:.2%}")
        print(f"  Hits:     {summary['memory']['hits']}")
        print(f"  Misses:   {summary['memory']['misses']}")
        
        print("\n🧬 Antibody Diversity:")
        print(f"  Current: {summary['antibody_diversity']['current']:.3f}")
        print(f"  Mean:    {summary['antibody_diversity']['mean']:.3f}")
        print(f"  Trend:   {summary['antibody_diversity']['trend']:+.3f}")
        
        if summary['by_threat_type']:
            print("\n📊 Performance by Threat Type:")
            for t_type, cm in summary['by_threat_type'].items():
                print(f"  {t_type}: F1={cm['f1']:.3f}, Prec={cm['precision']:.3f}, Rec={cm['recall']:.3f}")
        
        print("="*60)


class ResponseTimeAnalysis:
    """Analyze response time characteristics."""
    
    def __init__(self, times: List[float]):
        self.times = np.array(times)
    
    @property
    def mean(self) -> float:
        return float(np.mean(self.times))
    
    @property
    def std(self) -> float:
        return float(np.std(self.times))
    
    @property
    def percentiles(self) -> Dict[str, float]:
        return {
            'p50': float(np.percentile(self.times, 50)),
            'p90': float(np.percentile(self.times, 90)),
            'p95': float(np.percentile(self.times, 95)),
            'p99': float(np.percentile(self.times, 99)),
        }
    
    def memory_vs_novel_comparison(
        self,
        memory_times: List[float],
        novel_times: List[float]
    ) -> Dict:
        """Compare response times for memory hits vs novel threats."""
        return {
            'memory_mean': np.mean(memory_times) if memory_times else 0,
            'novel_mean': np.mean(novel_times) if novel_times else 0,
            'speedup': np.mean(novel_times) / np.mean(memory_times) if memory_times and np.mean(memory_times) > 0 else 0
        }
