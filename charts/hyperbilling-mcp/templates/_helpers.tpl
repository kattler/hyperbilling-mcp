{{- define "hyperbilling-mcp.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "hyperbilling-mcp.fullname" -}}
{{- if .Values.fullnameOverride -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- $name := default .Chart.Name .Values.nameOverride -}}
{{- if contains $name .Release.Name -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}
{{- end -}}

{{- define "hyperbilling-mcp.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
app.kubernetes.io/name: {{ include "hyperbilling-mcp.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{- define "hyperbilling-mcp.selectorLabels" -}}
app.kubernetes.io/name: {{ include "hyperbilling-mcp.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{- define "hyperbilling-mcp.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{- default (include "hyperbilling-mcp.fullname" .) .Values.serviceAccount.name -}}
{{- else -}}
{{- default "default" .Values.serviceAccount.name -}}
{{- end -}}
{{- end -}}

{{- define "hyperbilling-mcp.image" -}}
{{- $tag := .Values.image.tag | default .Chart.AppVersion -}}
{{- printf "%s/%s:%s" (.Values.image.registry | trimSuffix "/") .Values.image.repository $tag -}}
{{- with .Values.image.digest -}}
@{{ . }}
{{- end -}}
{{- end -}}

{{- define "hyperbilling-mcp.secretName" -}}
{{- default (include "hyperbilling-mcp.fullname" .) .Values.hyperbilling.existingSecret -}}
{{- end -}}

{{- define "hyperbilling-mcp.secretKey" -}}
{{- if .Values.hyperbilling.existingSecret -}}
{{- .Values.hyperbilling.existingSecretKey -}}
{{- else -}}
HYPERBILLING_CLIENT_SECRET
{{- end -}}
{{- end -}}

{{- define "hyperbilling-mcp.fallbackEnabled" -}}
{{- if or .Values.hyperbilling.existingSecret .Values.hyperbilling.clientSecret -}}true{{- end -}}
{{- end -}}
