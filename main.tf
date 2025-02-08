terraform {
    required_providers {
        aws = {
            source = "hashicorp/aws"
            version = "~> 5.86.0"
        }
        random = {
            source  = "hashicorp/random"
            version = "~> 3.0"
        }
        local = {
            source = "hashicorp/local"
            version = "~> 2.5"
        }
    }
    required_version = ">= 1.2.0"
}

# setup
variable "environment" {
    type = string
    default = "dev" # prod, staging, etc
}

variable "region" {
    type = string
    default = "ap-northeast-2"
}

data "local_file" "lambda_zip" {
    filename = "lambda.zip"
}

variable "openai_api_key" {
    type = string # TF_VAR_openai_api_key
}

provider "aws" {
    region = var.region
}

resource "random_id" "setup_tag" {
    byte_length = 4
}

# AWS S3 Configurations
resource "aws_s3_bucket" "datastore" {
    bucket = "paperscraper2-${var.environment}-${random_id.setup_tag.hex}"
    force_destroy = true
}

resource "aws_s3_bucket_versioning" "datastore_versioning" {
    bucket = aws_s3_bucket.datastore.id
    versioning_configuration {
        status = "Disabled"
    }
}

resource "aws_s3_bucket_public_access_block" "datastore_public_access" {
    bucket = aws_s3_bucket.datastore.id

    block_public_acls = true
    block_public_policy = true
    ignore_public_acls = true
    restrict_public_buckets = true
}

# AWS Lambda Configuration
resource "aws_iam_role" "scraper_lambda_role" {
    name = "paperscraper2-scraper-lambda-role-${var.environment}-${random_id.setup_tag.hex}"

    assume_role_policy = jsonencode({
        Version = "2012-10-17"
        Statement = [
            {
                Effect = "Allow"
                Principal = {
                    Service = "lambda.amazonaws.com"
                }
                Action = "sts:AssumeRole"
            }
        ]
    })
}

resource "aws_iam_role_policy_attachment" "scraper_lambda_basic_exec" {
    role = aws_iam_role.scraper_lambda_role.name
    policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_policy" "scraper_lambda_policy" {
    name = "paperscraper2-scraper-lambda-policy-${var.environment}-${random_id.setup_tag.hex}"
    description = "Custom policy for scraper lambda (s3 access)"

    policy = jsonencode({
        Version: "2012-10-17",
        Statement: [
            {
                Effect: "Allow",
                Action: "s3:ListAllMyBuckets",
                Resource: "arn:aws:s3:::*"
            },
            {
                Effect: "Allow",
                Action: "s3:*",
                Resource: [
                    "arn:aws:s3:::${aws_s3_bucket.datastore.bucket}",
                    "arn:aws:s3:::${aws_s3_bucket.datastore.bucket}/*"
                ]
            }
        ]
    })
}

resource "aws_iam_role_policy_attachment" "scraper_lambda_policy_attachment" {
    role = aws_iam_role.scraper_lambda_role.name
    policy_arn = aws_iam_policy.scraper_lambda_policy.arn
}

resource "aws_lambda_function" "scraper_lambda" {
    function_name = "paperscraper2-${var.environment}-${random_id.setup_tag.hex}"
    role = aws_iam_role.scraper_lambda_role.arn
    architectures = ["arm64"]
    runtime = "provided.al2023"
    handler = "bootstrap"
    filename = data.local_file.lambda_zip.filename
    source_code_hash = data.local_file.lambda_zip.content_sha1

    timeout = 120

    environment {
        variables = {
            BUCKET = aws_s3_bucket.datastore.bucket
            REGION = var.region
            OPENAI_API_KEY = var.openai_api_key
        }
    }
}

# AWS EventBridge Configuration
resource "aws_cloudwatch_event_rule" "scraper_lambda_cron" {
    name = "paperscraper2-scraper-lambda-cron-${var.environment}-${random_id.setup_tag.hex}"
    description = "Trigger Lambda function at 16:00 UTC (1am KST)"
    schedule_expression = "cron(0 16 * * ? *)"
}

resource "aws_cloudwatch_event_target" "invoke_lambda" {
    rule = aws_cloudwatch_event_rule.scraper_lambda_cron.name
    target_id = "lambda"
    arn = aws_lambda_function.scraper_lambda.arn
}

resource "aws_lambda_permission" "allow_eventbridge" {
    statement_id = "AllowExecutionFromEventBridge"
    action = "lambda:InvokeFunction"
    function_name = aws_lambda_function.scraper_lambda.function_name
    principal = "events.amazonaws.com"
    source_arn = aws_cloudwatch_event_rule.scraper_lambda_cron.arn
}

# Outputs
output "datastore_output" {
    value = aws_s3_bucket.datastore.id
}

output "scraper_lambda_output" {
    value = aws_lambda_function.scraper_lambda.function_name
}

output "scraper_lambda_cron_output" {
    value = aws_cloudwatch_event_rule.scraper_lambda_cron.name
}
