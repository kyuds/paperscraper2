terraform {
    required_providers {
        aws = {
            source = "hashicorp/aws"
            version = "~> 4.16"
        }
        random = {
            source  = "hashicorp/random"
            version = "~> 3.0"
        }
    }
    required_version = ">= 1.2.0"
}

variable "environment" {
    type = string
    default = "dev" # prod, staging, etc
}

provider "aws" {
    region = "ap-northeast-2"
}

resource "random_id" "bucket_suffix" {
    byte_length = 4
}

resource "aws_s3_bucket" "datastore" {
    bucket = "paperscraper2-${var.environment}-${random_id.bucket_suffix.hex}"
}

resource "aws_s3_bucket_versioning" "data_bucket_versioning" {
    bucket = aws_s3_bucket.datastore.id
    versioning_configuration {
        status = "Disabled"
    }
}
